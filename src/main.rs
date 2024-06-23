//cargo build --release --target=x86_64-unknown-linux-musl

use std::fs::File;
use std::io::BufReader;
use xml::common::Position;
use xml::reader::{ParserConfig, XmlEvent};
use std::collections::HashMap;
use std::process::Command;
use std::fs;
use gethostname::gethostname;

//function check speed 
fn speed_test(stream_type: &str, host: &str, port: i32, user: &str, pass: &str, dir: &str) -> String  {   
    
    //println!("{} {} {} {} {} {}", stream_type, host, port, user, pass, dir);

    
    //test sftp download
    let mut cmd = "curl -pks sftp://".to_owned()+ host +":"+ &port.to_string() + dir +"test --user '"+ user +"':'"+ pass +"' -T /app/sonic_speed/sonic_speed.test --ftp-create-dirs && curl -ksw %{speed_download} sftp://"+ host +":"+ &port.to_string() + dir +"test --user '"+ user +"':'"+ pass +"' -o /dev/null &"; 

    //test sftp upload
    if stream_type == "SFtpUpload" {
        cmd = "curl -pksw %{speed_upload} sftp://".to_owned()+ host +":"+ &port.to_string() + dir +" --user '"+ user +"':'"+ pass +"' -T /app/sonic_speed/sonic_speed.test --ftp-create-dirs &";    
        
    }


    let output = 
        Command::new("bash")
            .arg("-c")
            .arg(cmd)
            .output()
            .expect("failed to execute process");
    
            let speed = String::from_utf8_lossy(&output.stdout);           
            let speed_str: String = speed.to_string();
            return speed_str;

    
    }


fn main() {

      
    let file = File::open("/app/ftpUpload/lastCfg/workConfig").unwrap();
    let type_agent1 = "SFtpU";
    let type_agent2 = "SFtpD";

    let mut reader = ParserConfig::default()
        .ignore_root_level_whitespace(false)
        .create_reader(BufReader::new(file));

    let mut hash_map_filter:HashMap<String, i32> = HashMap::new();        

    loop {
        match reader.next() {
            Ok(e) => {

                match e {

                    XmlEvent::EndDocument => {
                        break;
                    }

                    XmlEvent::StartElement { name, attributes, .. } => {
            
                        if name.local_name.starts_with(type_agent1) || name.local_name.starts_with(type_agent2) {   
                        
                               let mut _string = name.to_string().to_owned();                                   
                            

                            for attr in attributes {
                                
                                match attr.name.local_name.as_str() {
                                    "ftpServer" | "ftpServerPort" | "ftpUser" | "ftpPassword" | "ftpSrcDir" | "ftpCacheDir"  
                                       => _string = _string  + ";" + &attr.value.to_string(),
                                     _ => (),
        
                                  }
                                 

                            }
                            //add to hashmap
                            //println!("{:?}", _string);
                            hash_map_filter.entry(_string).and_modify(|count| *count += 1).or_insert(1);


                    }
                        
                    },

                    _ => ()
                }
            }
            Err(e) => {
                eprintln!("Error at {}: {e}", reader.position());
                break;
            },
        }
    }


    for (key, value) in &hash_map_filter {

       // println!("{}:{}",key, value);

        let parts = key.split(";").collect::<Vec<_>>();
        let port = parts[2].parse::<i32>().expect("failed get port from vector");
        let dirs = parts[5];

        if parts[0] == "SFtpDownload"{ 
              let dirs = parts[5].to_owned() +  "test";                   
        }

        
    //check speed by constant size file
    let result: String = speed_test(parts[0], parts[1], port.try_into().unwrap(), parts[3], parts[4], dirs);
        let mut host = gethostname().into_string().expect("Unable gethostname").to_string();
                host = "q".to_owned() + &host;
        let influx_line: String = host.to_owned()+ "," + parts[0] + "=" + parts[1] + " bsec="+ &result + "\n";
        
           //println!("{}", influx_line);

    //write result to file
            fs::write("/app/sonic_speed/result/".to_owned() + parts[0] +"_"+ parts[1] +".txt", &influx_line).expect("Unable to write file");


    }





}