use std::{
    cell::LazyCell,
    collections::VecDeque,
    sync::{LazyLock, OnceLock},
    task::{Context, Poll, Waker},
    time::{Duration, Instant},
};
use std::{
    io::{BufRead, Write},
    thread::sleep,
};

use serialport::{SerialPortInfo, SerialPortType};

#[cfg(not(target_arch = "x86_64"))]
use blinkt::{Blinkt, BlinktSpi};
#[cfg(target_arch = "x86_64")]
use board::color_to_vec3;
use constants::*;
use ligth_point::LigthPoint;
#[cfg(target_arch = "x86_64")]
use macroquad::prelude::*;

use crate::{
    board::{Board, MachineStateChange, Scenario},
    product::ProductPlan,
    product::{Product, Step},
    time_manager::TimeManager,
    time_manager::VirtualInstant,
};

const BAUD_RATE: u32 = 115_200;

mod board;
mod constants;
mod ligth_point;
mod module;
mod product;
mod time_manager;

#[cfg(target_arch = "x86_64")]
#[macroquad::main("Board")]
async fn main() {
    main_inner().await;
}
#[cfg(not(target_arch = "x86_64"))]
fn main() {
    let future = std::pin::pin!(main_inner());
    let mut context = Context::from_waker(Waker::noop());
    match future.poll(&mut context) {
        Poll::Pending => {}
        Poll::Ready(()) => {}
    }
}

async fn main_inner() {
    #[cfg(target_arch = "x86_64")]
    board::Board::set_screen_size();
    let mut board = Board::new();

    #[cfg(not(target_arch = "x86_64"))]
    let mut blinkt = Blinkt::with_spi(
        BlinktSpi::with_settings(
            blinkt::spi::Bus::Spi1,
            blinkt::spi::SlaveSelect::Ss0,
            1_000_000,
            blinkt::spi::Mode::Mode0,
        )
        .unwrap(),
        X_NUM_MODULES * Y_NUM_MODULES * (7 + 6),
    );
    board.set_storage(STEPS_TOP_NORMAL.clone());
    board.set_storage(STEPS_BOTTOM_NORMAL.clone());
    board.set_scenario(BOTTOM_SUPPLY_DIFFICULTY.clone());

    let (speed_button, scenario_button) = init();

    let (Some(speed_button), Some(scenario_button)) = (speed_button, scenario_button) else {
        panic!("No speed and scenario button found");
    };

    let mut speed_button_port = serialport::new(speed_button.port_name, BAUD_RATE)
        .open()
        .unwrap();
    let mut scenario_button_port = serialport::new(scenario_button.port_name, BAUD_RATE)
        .open()
        .unwrap();

    let mut speed_button_reader =
        std::io::BufReader::new(speed_button_port.try_clone().unwrap()).lines();
    let mut scenario_button_reader =
        std::io::BufReader::new(scenario_button_port.try_clone().unwrap()).lines();

    loop {
        if let Some(Ok(speed)) = speed_button_reader.next() {
            println!("Speed: {}", speed);
        }
        if let Some(Ok(scenario)) = scenario_button_reader.next() {
            println!("Scenario: {}", scenario);
            match scenario
                .to_ascii_lowercase()
                .trim()
                .split_ascii_whitespace()
                .collect::<Vec<&str>>()
                .as_slice()
            {
                ["boot"] => {
                    speed_button_port.write_all(b"boot\n").unwrap();
                }
                ["scenario", scenario_num] => {
                    let message = format!("scenario {}\n", scenario_num);

                    speed_button_port.write_all(message.as_bytes()).unwrap();

                    match *scenario_num {
                        "1" => {
                            board.set_scenario(Scenario::starting_scenario().clone());
                        }
                        "2" => {
                            board.set_scenario(BOTTOM_SUPPLY_DIFFICULTY.clone());
                        }
                        "3" => {
                            board.set_scenario(MAINTENANCE.clone());
                        }
                        _ => {
                            println!("Invalid scenario number");
                        }
                    }
                }
                ["start"] => {
                    speed_button_port.write_all(b"start\n").unwrap();
                    for i in 0..11 {
                        let percentage = i as f32 * 0.1;
                        let message = format!("progress {}\n", percentage);
                        speed_button_port.write_all(message.as_bytes()).unwrap();
                        sleep(Duration::from_millis(500));
                    }
                }
                ["resume"] => {
                    speed_button_port.write_all(b"resume\n").unwrap();
                    board.time_manager.resume();
                }
                ["pause"] => {
                    speed_button_port.write_all(b"pause\n").unwrap();
                    board.time_manager.pause();
                }
                ["stop"] => {
                    speed_button_port.write_all(b"stop\n").unwrap();
                    board.set_scenario(Scenario::starting_scenario());
                }
                _ => {}
            }
        }

        let start_time = Instant::now();

        #[cfg(target_arch = "x86_64")]
        {
            // Handle keyboard input for time control
            handle_time_controls(&mut board.time_manager);
            clear_background(GRAY);
        }

        board.reset(LED_OFF_COLOR);

        board.update();

        #[cfg(not(target_arch = "x86_64"))]
        for (pixel, mut color) in blinkt.iter_mut().zip(board.colors()) {
            pixel.set_rgbb(
                (color.red * 255.0) as u8,
                (color.green * 255.0) as u8,
                (color.blue * 255.0) as u8,
                0.1,
            );
        }
        #[cfg(not(target_arch = "x86_64"))]
        blinkt.show().unwrap();

        #[cfg(target_arch = "x86_64")]
        {
            for key in get_keys_pressed() {
                match key {
                    KeyCode::Key7 => board.set_scenario(Scenario::starting_scenario().clone()),
                    KeyCode::Key8 => board.set_scenario(BOTTOM_SUPPLY_DIFFICULTY.clone()),
                    KeyCode::Key9 => board.set_scenario(MAINTENANCE.clone()),
                    _ => {}
                }
            }
            board.draw();
            // Draw speed indicator
            draw_speed_indicator(&board.time_manager, vec2(10.0, 10.0));
            next_frame().await
        }

        #[cfg(not(target_arch = "x86_64"))]
        std::thread::sleep(
            (start_time + Duration::from_secs(1) / 100).saturating_duration_since(Instant::now()),
        );
    }
}

fn init() -> (Option<SerialPortInfo>, Option<SerialPortInfo>) {
    let ports = serialport::available_ports().expect("no ports found");
    let mut speed_button = None;
    let mut scenario_button = None;
    for port in ports {
        let SerialPortType::UsbPort(ref usb_port_info) = port.port_type else {
            // Skip non-USB ports
            continue;
        };
        if usb_port_info.vid != 0x303A || usb_port_info.pid != 0x1001 {
            // Skip specific USB device
            continue;
        }

        println!("Found usb: {:?}", usb_port_info.product);
        // read_serial_to_std_out(port.clone());

        if usb_port_info.product == Some("Serielles USB-Gerät (COM5)".to_string()) {
            scenario_button = Some(port);
        } else {
            speed_button = Some(port)
        }
        // if usb_port_info.serial_number == Some("80:65:99:BD:21:5C".to_string()) {
        //     println!("Found usb: {:?}", port);
        //     speed_button = Some(port);
        // } else if usb_port_info.serial_number == Some("80:65:99:BD:1E:14".to_string()) {
        //     println!("Found usb: {:?}", port);
        //     scenario_button = Some(port);
        // }
    }
    (speed_button, scenario_button)
}

fn read_serial_to_std_out(serial_port_info: SerialPortInfo) {
    let port = serialport::new(serial_port_info.port_name, BAUD_RATE)
        // .
        .open()
        .expect("Failed to open serial_port");

    for line in std::io::BufReader::new(port).lines() {
        if line.is_err() {
            // println!("Error reading line: {:?}", line.err());
            continue;
        }
        println!("{}", line.unwrap());
        std::io::stdout().flush().unwrap();
    }
}

#[cfg(target_arch = "x86_64")]
/// Handle keyboard input for time control
fn handle_time_controls(time_manager: &mut TimeManager) {
    for key in get_keys_pressed() {
        match key {
            KeyCode::Key1 => time_manager.set_speed(0.25), // Quarter speed
            KeyCode::Key2 => time_manager.set_speed(0.5),  // Half speed
            KeyCode::Key3 => time_manager.set_speed(1.0),  // Normal speed
            KeyCode::Key4 => time_manager.set_speed(2.0),  // Double speed
            KeyCode::Key5 => time_manager.set_speed(4.0),  // Quadruple speed
            KeyCode::Key6 => time_manager.set_speed(8.0),  // 8x speed
            KeyCode::Up => {
                // fine speed adjustment
                let current_speed = time_manager.speed();
                time_manager.set_speed((current_speed * 1.25).min(10.0));
            }
            KeyCode::Down => {
                let current_speed = time_manager.speed();
                time_manager.set_speed((current_speed * 0.8).max(0.1));
            }
            _ => {}
        }
    }
}

#[cfg(target_arch = "x86_64")]
/// Draw speed indicator and controls help
fn draw_speed_indicator(time_manager: &TimeManager, position: Vec2) {
    let speed = time_manager.speed();
    // Status display
    draw_text(
        &format!("Speed: {speed:.2}x"),
        position.x,
        position.y,
        24.0,
        GREEN,
    );

    // Virtual time display (formatted)
    let virtual_time_text = format!("Virtual Time: {}", time_manager.format_time());
    draw_text(
        &virtual_time_text,
        position.x,
        position.y + 25.0,
        20.0,
        LIGHTGRAY,
    );

    // Controls help
    let help_text = &[
        "Controls:",
        "1-6: Set speed (0.25x - 8x)",
        "↑/↓: Fine adjust speed",
        "Space: Pause/Resume",
        "R: Reset time",
    ];

    for (i, line) in help_text.iter().enumerate() {
        draw_text(
            line,
            position.x,
            position.y + 60.0 + i as f32 * 18.0,
            16.0,
            GRAY,
        );
    }
}
