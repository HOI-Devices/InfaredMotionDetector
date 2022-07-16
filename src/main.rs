extern crate tungstenite;
extern crate url;
extern crate chrono;
extern crate gpio;
extern crate ansi_term;
#[macro_use]
extern crate serde_json;

mod client;
mod gpio_handler;
mod tests;
mod console_logger;

use chrono::{DateTime, TimeZone, NaiveDateTime, Utc};
use std::sync::{Arc, Mutex};
use gpio_handler::GpioHandler;
use client::Client;
use std::thread;
use std::thread::JoinHandle;
use std::thread::sleep;
use std::time::Duration;

fn start_gpio_thread(
        pin:u16,
        times_sensed:&Arc<Mutex<i8>>,
        utc:&Arc<Mutex<DateTime<Utc>>>,
        running:&Arc<Mutex<bool>>)-> JoinHandle<()> {
    let times_sensed = Arc::clone(times_sensed);
    let utc = Arc::clone(utc);
    let running = Arc::clone(running);
    let mut gpio_handler = GpioHandler::new(pin);

    let handle:JoinHandle<()> = thread::spawn(move || {
        gpio_handler.begin_monitoring(&times_sensed,&utc,&running);
    });
    return handle;
}

fn main() {
    let mut client = Client::new(
        "192.168.1.109".to_string(),
        "50223".to_string(),
        "".to_string(),
        "test".to_string(),
        "infared".to_string(),
        "test".to_string());

    let mut encountered_error = false;
    let utc: Arc<Mutex<DateTime<Utc>>> =Arc::new(Mutex::new(Utc::now()));    
    let times_sensed:Arc<Mutex<i8>> = Arc::new(Mutex::new(0));
    let mut running = Arc::new(Mutex::new(true));

    //begin monitoring both sensors and checking with server
    let handle1 =  start_gpio_thread(24,&times_sensed,&utc,&running);   
    let handle2 = start_gpio_thread(23,&times_sensed,&utc,&running);
 
    while true{
        client.begin_monitoring(&mut encountered_error,&utc,&times_sensed);
        if encountered_error == false{
            *running.lock().unwrap() = false;
            break;
        }
        thread::sleep(Duration::from_millis(10000));
    }    

    handle1.join().unwrap();
    handle2.join().unwrap();
}