use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::info;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

mod audio;
mod config;
mod error;
mod model;
mod output;
mod state;
mod transcribe;

/// Maximum recording duration in toggle mode (5 minutes)
const TOGGLE_MODE_TIMEOUT_SECS: u32 = 300;

#[derive(Parser)]
#[command(name = "dev-voice")]
#[command(about = "Voice dictation for Linux developers")]
#[command(version)]
struct Cli {
    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start or stop voice recording (toggle mode when duration=0)
    Start {
        /// Override model path
        #[arg(short, long)]
        model: Option<String>,

        /// Recording duration in seconds (0 = toggle mode)
        #[arg(short, long, default_value = "0")]
        duration: u32,

        /// Copy to clipboard instead of typing
        #[arg(short, long)]
        clipboard: bool,
    },

    /// Stop a running recording
    Stop,

    /// Download a whisper model
    Download {
        /// Model size: tiny.en, base.en, small.en, medium.en, large
        #[arg(default_value = "base.en")]
        model: String,
    },

    /// Show or edit configuration
    Config {
        /// Print config file path
        #[arg(long)]
        path: bool,

        /// Reset to default configuration
        #[arg(long)]
        reset: bool,
    },

    /// Check system dependencies
    Doctor,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging with both console and file output
    init_logging(cli.verbose)?;

    match cli.command {
        Commands::Start { model, duration, clipboard } => {
            cmd_start(model, duration, clipboard)?;
        }
        Commands::Stop => {
            cmd_stop()?;
        }
        Commands::Download { model } => {
            cmd_download(&model)?;
        }
        Commands::Config { path, reset } => {
            cmd_config(path, reset)?;
        }
        Commands::Doctor => {
            cmd_doctor()?;
        }
    }

    Ok(())
}

/// Initialize logging with console and file output
fn init_logging(verbose: bool) -> Result<()> {
    let filter = if verbose { "debug" } else { "info" };

    // Set up file logging
    let log_dir = state::get_log_dir()?;
    let file_appender = RollingFileAppender::new(Rotation::DAILY, log_dir, "dev-voice.log");

    // Create layers
    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(file_appender)
        .with_ansi(false)
        .with_target(false);

    let console_layer = tracing_subscriber::fmt::layer()
        .with_target(false);

    // Combine layers
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(filter))
        .with(console_layer)
        .with(file_layer)
        .init();

    Ok(())
}

fn cmd_start(model_override: Option<String>, duration: u32, clipboard: bool) -> Result<()> {
    // Check if toggle mode (duration = 0)
    if duration == 0 {
        return cmd_start_toggle(model_override, clipboard);
    }

    // Fixed duration mode
    cmd_start_fixed(model_override, duration, clipboard)
}

/// Toggle mode: first call starts, second call stops
fn cmd_start_toggle(model_override: Option<String>, clipboard: bool) -> Result<()> {
    // Check if already recording
    if let Some(recording_state) = state::is_recording()? {
        info!("Recording in progress, sending stop signal...");
        state::stop_recording(&recording_state)?;
        println!("Stopping recording...");
        return Ok(());
    }

    // Start new recording
    info!("Starting toggle mode recording (max {} seconds)", TOGGLE_MODE_TIMEOUT_SECS);
    println!("Recording started. Run 'dev-voice start' again or 'dev-voice stop' to finish.");

    // Set up signal handler and mark as recording
    state::toggle::setup_signal_handler()?;
    state::toggle::start_recording()?;

    // Ensure cleanup on exit
    let _cleanup = scopeguard::guard((), |_| {
        let _ = state::toggle::cleanup_recording();
    });

    // Load config and run recording
    let mut cfg = config::load()?;
    if let Some(model_path) = model_override {
        cfg.model.path = model_path.into();
    }

    if !cfg.model.path.exists() {
        anyhow::bail!(
            "Model not found: {}\nRun: dev-voice download {}",
            cfg.model.path.display(),
            cfg.model.path.file_stem().unwrap_or_default().to_string_lossy()
        );
    }

    let display_server = output::DisplayServer::detect();
    let output_mode = if clipboard {
        output::OutputMode::Clipboard
    } else {
        output::OutputMode::Type
    };

    info!("Loading whisper model...");
    let transcriber = transcribe::Transcriber::new(&cfg.model.path)?;
    info!("Model loaded successfully");

    // Capture audio with toggle mode (checks for stop signal)
    info!("Listening... (press Ctrl+C or run 'dev-voice stop' to finish)");
    let audio_data = audio::capture_toggle(TOGGLE_MODE_TIMEOUT_SECS, cfg.audio.sample_rate)?;
    info!("Captured {} samples", audio_data.len());

    if audio_data.is_empty() {
        info!("No audio captured");
        return Ok(());
    }

    // Create processing state file
    let processing_file = state::get_state_dir()?.join("processing");
    std::fs::write(&processing_file, "")?;
    let _processing_cleanup = scopeguard::guard((), |_| {
        let _ = std::fs::remove_file(&processing_file);
    });

    // Transcribe
    info!("Transcribing...");
    let text = transcriber.transcribe(&audio_data)?;

    if text.is_empty() {
        info!("No speech detected");
        return Ok(());
    }

    info!("Transcribed: {}", text);
    output::output_text(&text, output_mode, &display_server)?;
    info!("Text output via {:?}", output_mode);

    // Send notification with preview
    let preview = if text.len() > 80 {
        format!("{}...", text.chars().take(77).collect::<String>())
    } else {
        text
    };
    send_notification("Transcription Complete", &preview, "normal");

    Ok(())
}

/// Fixed duration recording mode
fn cmd_start_fixed(model_override: Option<String>, duration: u32, clipboard: bool) -> Result<()> {
    info!("Loading configuration...");
    let mut cfg = config::load()?;

    if let Some(model_path) = model_override {
        cfg.model.path = model_path.into();
    }

    info!("Model: {}", cfg.model.path.display());

    if !cfg.model.path.exists() {
        anyhow::bail!(
            "Model not found: {}\nRun: dev-voice download {}",
            cfg.model.path.display(),
            cfg.model.path.file_stem().unwrap_or_default().to_string_lossy()
        );
    }

    let display_server = output::DisplayServer::detect();
    info!("Display server: {:?}", display_server);

    let output_mode = if clipboard {
        output::OutputMode::Clipboard
    } else {
        output::OutputMode::Type
    };
    info!("Output mode: {:?}", output_mode);

    info!("Loading whisper model...");
    let transcriber = transcribe::Transcriber::new(&cfg.model.path)?;
    info!("Model loaded successfully");

    info!("Recording for {} seconds...", duration);
    let audio_data = audio::capture(duration, cfg.audio.sample_rate)?;
    info!("Captured {} samples", audio_data.len());

    // Create processing state file
    let processing_file = state::get_state_dir()?.join("processing");
    std::fs::write(&processing_file, "")?;
    let _processing_cleanup = scopeguard::guard((), |_| {
        let _ = std::fs::remove_file(&processing_file);
    });

    info!("Transcribing...");
    let text = transcriber.transcribe(&audio_data)?;

    if text.is_empty() {
        info!("No speech detected");
        return Ok(());
    }

    info!("Transcribed: {}", text);
    output::output_text(&text, output_mode, &display_server)?;
    info!("Text output via {:?}", output_mode);

    // Send notification with preview
    let preview = if text.len() > 80 {
        format!("{}...", text.chars().take(77).collect::<String>())
    } else {
        text
    };
    send_notification("Transcription Complete", &preview, "normal");

    Ok(())
}

/// Stop a running recording
fn cmd_stop() -> Result<()> {
    if let Some(recording_state) = state::is_recording()? {
        info!("Stopping recording (PID: {})", recording_state.pid);
        state::stop_recording(&recording_state)?;
        println!("Stop signal sent to recording process");
    } else {
        println!("No recording in progress");
    }
    Ok(())
}

fn cmd_download(model_name: &str) -> Result<()> {
    let cfg = config::load()?;
    let models_dir = cfg.model.path.parent().unwrap_or(std::path::Path::new("."));

    let model_info = model::ModelInfo::find(model_name).ok_or_else(|| {
        let available = model::ModelInfo::available_models();
        anyhow::anyhow!(
            "Unknown model: {}\nAvailable models: {}",
            model_name,
            available.join(", ")
        )
    })?;

    let dest = model::download_model(model_info, models_dir)?;
    info!("Model ready: {}", dest.display());

    Ok(())
}

fn cmd_config(show_path: bool, reset: bool) -> Result<()> {
    if reset {
        let cfg = config::Config::default();
        config::save(&cfg)?;
        info!("Configuration reset to defaults");
        return Ok(());
    }

    if show_path {
        let path = config::config_path()?;
        println!("{}", path.display());
        return Ok(());
    }

    let cfg = config::load()?;
    let toml = toml::to_string_pretty(&cfg)?;
    println!("{}", toml);

    Ok(())
}

/// Send desktop notification
fn send_notification(title: &str, body: &str, urgency: &str) {
    let _ = std::process::Command::new("notify-send")
        .args([
            "-a", "dev-voice",
            "-i", "audio-input-microphone",
            "-u", urgency,
            title,
            body,
        ])
        .spawn();
}

fn cmd_doctor() -> Result<()> {
    println!("Checking system dependencies...\n");

    let wtype_ok = std::process::Command::new("which")
        .arg("wtype")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    println!(
        "[{}] wtype (Wayland text injection)",
        if wtype_ok { "OK" } else { "MISSING" }
    );

    let xdotool_ok = std::process::Command::new("which")
        .arg("xdotool")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    println!(
        "[{}] xdotool (X11 text injection)",
        if xdotool_ok { "OK" } else { "MISSING" }
    );

    let display = output::DisplayServer::detect();
    println!("\nDisplay server: {:?}", display);

    match display {
        output::DisplayServer::Wayland if !wtype_ok => {
            println!("\nWARNING: You're on Wayland but wtype is not installed.");
            println!("Install with: sudo dnf install wtype");
        }
        output::DisplayServer::X11 if !xdotool_ok => {
            println!("\nWARNING: You're on X11 but xdotool is not installed.");
            println!("Install with: sudo dnf install xdotool");
        }
        _ => {}
    }

    let cfg = config::load()?;
    let model_ok = cfg.model.path.exists();
    println!(
        "\n[{}] Whisper model: {}",
        if model_ok { "OK" } else { "MISSING" },
        cfg.model.path.display()
    );

    if !model_ok {
        println!("\nDownload a model with: dev-voice download base.en");
    }

    let pw_ok = std::process::Command::new("pw-cli")
        .arg("info")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    println!(
        "\n[{}] PipeWire",
        if pw_ok { "OK" } else { "MISSING" }
    );

    // Show log location
    if let Ok(log_dir) = state::get_log_dir() {
        println!("\nLogs: {}", log_dir.display());
    }

    println!();
    Ok(())
}
