use gtk::prelude::*;
use std::fs;
use std::thread;
use std::time::{Duration, Instant};
use zmq;
use std::os::unix::fs::OpenOptionsExt;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "ByeUSBMsg", version, about)]
struct Args {
    #[arg(short = 'f', long = "file", default_value_t = "/var/lib/byeusb/monitoring.pause".to_string())]
    file: String,
}
fn create_pause_file(pause_file: &String) -> std::io::Result<()> {
    let _ = fs::remove_file(pause_file);
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(pause_file)?;
    Ok(())
}

// const PAUSE_FILE: &str = "/var/lib/byeusb/monitoring.pause";
const PAUSE_SECS: u64 = 5;

fn is_root() -> bool {
    unsafe { libc::geteuid() == 0 }
}

fn send(socket: &zmq::Socket, msg: &str) {
    if let Err(e) = socket.send(msg, 0) {
        eprintln!("[ZMQ] tx failed {}", e);
    }
}

fn main() {
    if !is_root() {
        eprintln!("[ERROR] ByeUSBMsg must be run as root.");
        std::process::exit(1);
    }
    
    if gtk::init().is_err() {
        eprintln!("[ERROR] failed to init gtk");
        return;
    }
    let args = Args::parse();
    let pause_file_close = args.file.clone();
    let ctx = zmq::Context::new();
    let pusher = match ctx.socket(zmq::PUSH) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[ERROR] failed to create PUSH socket: {}", e);
            return;
        }
    };
    if let Err(e) = pusher.connect("tcp://127.0.0.1:15641") {
        eprintln!("[ERROR] failed to connect PUSH: {}", e);
        return;
    }

    thread::sleep(Duration::from_millis(300));

    if let Err(e) = create_pause_file(&args.file) {
        eprintln!("[ERROR] failed to create pause file: {}", e);
        return;
    }

    send(&pusher, "stop");

    let window = gtk::Window::new(gtk::WindowType::Toplevel);
    window.set_title("ByeUSB monitoring paused");
    window.set_default_size(320, 100);
    window.set_position(gtk::WindowPosition::Center);
    window.set_keep_above(true);

    let label = gtk::Label::new(Some("ByeUSB paused"));
    window.add(&label);
    window.show_all();

    let start = Instant::now();
    let label_clone = label.clone();

    let pusher = std::cell::RefCell::new(pusher);

    glib::timeout_add_local(Duration::from_millis(100), move || {
        let elapsed = start.elapsed().as_secs();
        if elapsed >= PAUSE_SECS {
            let _ = fs::remove_file(&args.file);
            send(&pusher.borrow(), "start");
            gtk::main_quit();
            return glib::ControlFlow::Break;
        }
        let remaining = PAUSE_SECS - elapsed;
        label_clone.set_text(&format!("ByeUSB paused for {}s", remaining));
        glib::ControlFlow::Continue
    });

    gtk::main();
    let _ = fs::remove_file(pause_file_close);
}
