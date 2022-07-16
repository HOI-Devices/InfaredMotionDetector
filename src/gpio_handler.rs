use gpio::{GpioIn, GpioOut,GpioValue};
use client::Client;
use std::sync::{Arc, Mutex};
use std::thread;

use chrono::{DateTime, TimeZone, NaiveDateTime, Utc};
pub struct GpioHandler{
    input_pin : u16
}

impl GpioHandler{
    pub fn new(input: u16)->Self{
        Self{
            input_pin:input
        }
    }

    pub fn begin_monitoring(
            &self,times_open:&Arc<Mutex<i8>>,
            last_time_open:&Arc<Mutex<DateTime<Utc>>>,
            running:&Arc<Mutex<bool>>){

        let mut gpio_pin = gpio::sysfs::SysFsGpioInput::open(self.input_pin).unwrap();
       
        //deference the mutex to get the state of the application
        println!("Sensor Pin({}) Activated!!\n",self.input_pin);
        while true{
            let state = gpio_pin.read_value().unwrap();
            let running_status = *running.lock().unwrap();
            if running_status == false{
                break
            }
            if state == GpioValue::High{
                let mut last_time_data = last_time_open.lock().unwrap();
                *last_time_data = Utc::now();
                let mut times_data = times_open.lock().unwrap();
                if *times_data >70{
                    *times_data = 1;
                }
                else{
                    *times_data +=1;
                }
            }

        }
    }
}