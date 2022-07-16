

/*
Note the following tests assume
1.There is a matching server with the correct protocol
2.There are no network errors
*/

#[cfg(test)]
mod tests{
use std::thread::sleep;
use std::time::Duration;
use tungstenite::{connect, Message,WebSocket};
use tungstenite::client::AutoStream;
use url::Url;
use chrono::{DateTime, TimeZone, NaiveDateTime, Utc};
use std::env;
use serde_json::Value;
use std::fs;
use std::sync::{Arc, Mutex};
use std::thread;
use client::Client;

    #[test]
    fn test_successful_auth(){
        let mut client = Client::new(
            "192.168.1.109".to_string(),
            "50223".to_string(),
            "".to_string(),
            "no".to_string(),
            "infared".to_string(),
            "test".to_string());
        let url = format!("ws://{}:{}","192.168.1.109".to_string(),"50223".to_string());
        let (mut socket, response) =connect(Url::parse(&url).unwrap()).unwrap();
        assert_eq!(client.authenticate(&mut socket),true);
    }

    #[test]
    fn test_failed_auth(){
        let mut client = Client::new(
            "192.168.1.109".to_string(),
            "50223".to_string(),
            "2".to_string(),
            "no".to_string(),
            "infared".to_string(),
            "test".to_string());
        let url = format!("ws://{}:{}","192.168.1.109".to_string(),"50223".to_string());
        let (mut socket, response) =connect(Url::parse(&url).unwrap()).unwrap();
        assert_eq!(client.authenticate(&mut socket),false);
    }

    #[test]
    //failed auth triggers encountered error(passing the wrong password `2`)
    fn test_encountered_error(){
        let mut client = Client::new(
            "192.168.1.109".to_string(),
            "50223".to_string(),
            "2".to_string(),
            "no".to_string(),
            "infared".to_string(),
            "test".to_string());
        let mut encountered_error = false;
        let utc: Arc<Mutex<DateTime<Utc>>> =Arc::new(Mutex::new(Utc::now()));    
        let times_sensed:Arc<Mutex<i8>> = Arc::new(Mutex::new(0));
        client.begin_monitoring(&mut encountered_error,&utc,&times_sensed);
        assert_eq!(encountered_error,true);
    }
    


}