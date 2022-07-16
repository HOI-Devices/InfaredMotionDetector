use tungstenite::{connect, Message,WebSocket};
use tungstenite::client::AutoStream;
use url::Url;
use chrono::{DateTime, TimeZone, NaiveDateTime, Utc};
use std::env;
use serde_json::Value;
use std::fs;
use std::sync::{Arc, Mutex};
use std::thread;
use console_logger::ConsoleLogger;


pub struct Client{
    host:String,
    port:String,
    password:String,
    name:String,
    device_type:String,
    server_name:String,
    snapshot_last_times_sensed:i8, // used to know if we need to send a new alert
    logger:ConsoleLogger
}

impl  Client{
    pub fn new(host_data:String,
        port_data:String,
        pass:String, 
        device_name:String,
        type_of_bot:String,
        outside_server_name:String)->Self{

        Self{
            host:host_data.to_string(),
            port:port_data,
            password:pass,
            name:device_name,
            device_type:type_of_bot,
            server_name:outside_server_name,
            snapshot_last_times_sensed:0,
            logger: ConsoleLogger::new()
        }
    }

    pub fn authenticate(&mut self, socket:&mut WebSocket<AutoStream>)->bool{
        let send_password_result = socket.write_message(Message::Text(self.password.to_owned()));

        //send password
        if send_password_result.is_ok(){
            let send_name_and_type_result =
                socket.write_message(Message::Text(self.name_and_type()));
            //send name and type    
            if send_name_and_type_result.is_ok(){
                let send_server_name_result = 
                    socket.write_message(Message::Text(self.server_name.to_owned()));
                    // send server name
                if send_server_name_result.is_ok(){
                    return self.check_auth_response(socket);
                }
            }
        }
        return false;
    }

    fn check_auth_response(&mut self,  socket:&mut WebSocket<AutoStream>)-> bool{
        let msg_result = socket.read_message();

        if msg_result.is_ok(){
            let msg = msg_result.unwrap().into_text().unwrap();
            if msg == "success"{
                self.logger.log_basic_row("Successfully Authenticated!!\n","green");
                return true;
            }
            else{
                self.logger.log_failed_auth();
                return false;
            }
        }
        else{
            self.logger.log_failed_auth();
            return false;
        }
    }

     //keep listening for server requests and route the requests
    fn enter_main_loop(&mut self, 
        encountered_error:&mut bool, 
        socket:&mut WebSocket<AutoStream>,
        motion_last_sensed:&Arc<Mutex<DateTime<Utc>>>,
        number_of_times_sensed:&Arc<Mutex<i8>>){
        loop {
            let msg_result = socket.read_message();
            if msg_result.is_ok(){
                let msg = msg_result.unwrap().into_text().unwrap();
                if msg == "disconnect"{
                    let response_msg_result = socket.write_message(Message::Text("success".into()));
                    if response_msg_result.is_ok(){
                        *encountered_error = false;
                        break;
                    }
                }
                else{
                    self.route_message(msg,socket,motion_last_sensed,number_of_times_sensed);
                }
            }
            else{
                self.logger.log_error_encounter();
                *encountered_error = true;
                break;
            }
        }
    }

    pub fn begin_monitoring(&mut self,
        encountered_error: &mut bool, 
        motion_last_sensed:&Arc<Mutex<DateTime<Utc>>>,
        number_of_times_sensed:&Arc<Mutex<i8>>
         ){
        self.logger.log_welcome();
        let url = format!("ws://{}:{}",self.host,self.port);
        let attempt = connect(Url::parse(&url).unwrap());

        if attempt.is_ok(){
            let (mut socket, response) = attempt.unwrap();
            //if we successfully authenticated
            if self.authenticate(&mut socket) == true{
                self.enter_main_loop(encountered_error,&mut socket,motion_last_sensed,
                    number_of_times_sensed);
            }
            else{
                self.logger.log_failed_auth();
                self.logger.log_error_encounter();
                *encountered_error = true;
            }
        }
        else{
            self.logger.log_error_encounter();
            *encountered_error = true;
        }
    }

    fn route_message(&mut self,message:String,socket:&mut WebSocket<AutoStream>,
        motion_last_sensed:&Arc<Mutex<DateTime<Utc>>>,
        number_of_times_sensed:&Arc<Mutex<i8>>){
        if message =="deactivate"{
            let send_deactivate_status_result = socket.write_message(Message::Text("success".into()));
               // successfully notified the server of the success
            if send_deactivate_status_result.is_ok(){
                self.logger.log_basic_row("Deactivating Device!","red");
                self.enter_deactivation_loop(socket);
                self.logger.log_basic_row("Activated Device!","green");
            }
        }
        else if message == "alert"{
            self.check_for_and_send_alert(socket,number_of_times_sensed,motion_last_sensed);
        }
        else if message == "basic_data"{
            let last_time_data = *motion_last_sensed.lock().unwrap();
            let times_sensed_data = *number_of_times_sensed.lock().unwrap();
            socket.write_message(
                Message::Text(
                    self.formatted_basic_data(&times_sensed_data,&last_time_data)));
        }
    }

    fn enter_deactivation_loop(&mut self, socket:&mut WebSocket<AutoStream>){
        loop{
            let msg_result = socket.read_message();

            // if the msg result is valid
            if msg_result.is_ok(){
                let msg = msg_result.unwrap();
                let msg_text_result = msg.into_text();
                if msg_text_result.is_ok(){
    
                    let msg_text = msg_text_result.unwrap();
                    if msg_text == "activate"{
                        let send_activate_status_result = 
                            socket.write_message(Message::Text("success".into()));
                        // successfully notified the server of the success
                        if send_activate_status_result.is_ok(){
                            break;
                        };
                    };
                };
            }
            //connection error
            else{
                break;
            }
        }
    }

    fn check_for_and_send_alert(&mut self, 
        socket:&mut WebSocket<AutoStream>,
        number_of_times_sensed:&Arc<Mutex<i8>>,
        motion_last_sensed:&Arc<Mutex<DateTime<Utc>>>){
        //if the snapshot data != the current number of times sensed, there was new motion detected
        let snapshot_times_sensed  = *number_of_times_sensed.lock().unwrap();
        if snapshot_times_sensed != self.snapshot_last_times_sensed{
            self.snapshot_last_times_sensed = snapshot_times_sensed;
            self.logger.log_basic_row(" Server Alert Check Request - Motion Has Sensed!!","red");
            socket.write_message(Message::Text(self.alert(number_of_times_sensed,motion_last_sensed)));
        }
        else{
            socket.write_message(Message::Text(self.no_alert()));
        }
    }

    fn name_and_type(&mut self)-> String{
        let name_and_type = json!({
            "name":&self.name,
            "type":&self.device_type
        });
        return name_and_type.to_string();
    }
    
    fn alert(&mut self,number_of_times_sensed:&Arc<Mutex<i8>>,motion_last_sensed:&Arc<Mutex<DateTime<Utc>>>)->String{
        let alert_data = json!(
            {"status":"alert_present",
            "message":format!("{}(Infared motion sensor) sensed motion!!",self.name)});
        return alert_data.to_string();

    }

    fn no_alert(&mut self)->String{
        let no_alert_data = json!({"status":"no_alert_present"});
        return no_alert_data.to_string();
    }

    fn formatted_basic_data(&mut self,times_sensed:&i8, last_sensed:&DateTime<Utc>)->String{
        let formatted = json!({"last_sensed":last_sensed.to_string(), "times_sensed":times_sensed}).to_string();
        return formatted;
    }
}
