//
// Copyright 2019 Sͬeͥbͭaͭsͤtͬian
//
// Redistribution and use in source and binary forms, with or without modification, 
// are permitted provided that the following conditions are met:
//
// 1. Redistributions of source code must retain the above copyright notice, this 
// list of conditions and the following disclaimer.
//
// 2. Redistributions in binary form must reproduce the above copyright notice, 
// this list of conditions and the following disclaimer in the documentation and/or 
// other materials provided with the distribution.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND 
// ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED 
// WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED. 
// IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, 
// INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT 
// NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR 
// PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, 
// WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) 
// ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE 
// POSSIBILITY OF SUCH DAMAGE.
//

use std::io::prelude::*;
use std::net::TcpStream;
use std::net::TcpListener;
use std::fs;
use std::process::Command;
use std::env;
use std::path::Path;

// Let's go
fn main () {
  let path = env::current_dir();
  let path_str = path.unwrap().into_os_string().into_string();

  let get_ip_exec = path_str.unwrap() + &"/get_IP";
  let mut ip = execute_process (get_ip_exec);
  ip = ip.trim().to_string();

  ip = ip + &":80";
  //DEBUG println! ("IP: >{}<",ip);

  let listener = TcpListener::bind (ip).unwrap(); 

  for stream in listener.incoming () {
    let stream = stream.unwrap();

    handle_connection(stream);
  }
}

// Welcome Wo(Men), nice to meet you.
fn handle_connection (mut stream: TcpStream) {
  let mut buffer = [0;512];

  stream.read(&mut buffer).unwrap();
 
  let mut uri_path = get_path (String::from_utf8(buffer.to_vec()).unwrap());

  // normalize path, a simple path
  // a lonly slash is the index.shtml file
  let path = env::current_dir();
  let path_str = path.unwrap().into_os_string().into_string();
  if "/" == uri_path {
    uri_path = uri_path + &"index.shtml";
  }
  // a slash at beginning need previous path
  if uri_path.starts_with ("/") {
    uri_path = format!("{}{}", path_str.unwrap(),uri_path);
  }
  
  //DEBUG println! ("URI requested: {}", uri_path); 

  let mut response = format!("HTTP/1.1 404 NOT FOUND\n\rContent-Type: text/html;charset=utf-8\n\r\n\r{}","<html><head><title>Somewhere over the rainbow...</title></head><body>...blue birds fly</body></html>");

  let path_object_string = format! ("{}", uri_path);
  let path_object = Path::new(&path_object_string);

  if path_object.exists() {
    let contents = replace_ssi_command (fs::read_to_string(uri_path).unwrap());
    response = format!("HTTP/1.1 200 OK\n\rContent-Type: text/html;charset=utf-8\n\r\n\r{}",contents);
  }

  stream.write (response.as_bytes()).unwrap();
  stream.flush().unwrap();

}

// How can I help you?
fn get_path (http_header : String) -> String {
  //GET / HTTP/1.1
  
  let lines: Vec<&str> = http_header.split("\r\n").collect();
  for (pos, line) in lines.iter().enumerate() {
    if 0 == pos {
      //DEBUG println! ("LINE {}: {:#?}",pos,line);
      let token_line: Vec<&str> = line.split(" ").collect();
      for (pos_in_line, token) in token_line.iter().enumerate() {
        match pos_in_line {
          0 => {
            let _http_method = token;
          }
	  1 => {
	    let http_uri = token;
            //DEBUG println! ("URI >{}< requested", http_uri);
            return format! ("{}",http_uri);
          }
          _ => {
            let _ignored = token;
          }
        }
      }
    }
  }
  return format!("Yes we can!");
}

// My pleasure!
fn replace_ssi_command (to_parse : String) -> String {
  let mut result = "".to_string() ;
  {
    // looking for start and split
    let tokens: Vec<&str> = to_parse.split("<!--").collect();
    for (_pos, comment) in tokens.iter().enumerate() {

       const SSI_STATEMENT : char = '#';
       match comment.chars().next() {
          // OK we found an SSI statement 
          Some(SSI_STATEMENT) => {
            let tokens: Vec<&str> = comment.split("-->").collect();
            for (pos, cmd) in tokens.iter().enumerate() {
              if 0 == pos {
                let ssi_result = do_ssi_command (cmd.to_string());   
                //println! ("I found:{}", ssi_result);
                result = result + &ssi_result;
              }
              else {
                result = result + &cmd;
              }
            }
          },
          // do it for all other (not) matches
          _ => {
            result = result + &comment;
          },
       };
    }
  }
  return result;
}

// For you? I execute all my love!
fn do_ssi_exec (params : &Vec<&str>) -> String {
 // normalize... (1.=#exec, 2.=cmd[.], 3.=.)
//  log (info, "Exec parameter: {:#?} ");

  let mut contained : char = ' ';
  let mut container = String::from("");
  for (pos, token) in params.iter().enumerate() {
    if 0 < pos {
      if token.ends_with("=") {
        contained = '=';
        container = container + &token;
      }
      else { 
        if '=' == contained {
          contained = ' ';
          contained = token.get(..1).unwrap().chars().next().unwrap();
          container = container + &token;
        }
        else {
          if ' ' == contained {
            container = container + &token;
          }
          else {
            container = container + &" " + &token;
            if token.ends_with (contained) {
              contained = ' ';
            }
          }
        }
      }
     //DEBUG println! ("normalizes cmd: {}",container);
    }
  }


  if 0 < params.len() {
    let tokens: Vec<&str> = container.split("=").collect();
    let mut command : String = "".to_string();   
    for (pos, token) in tokens.iter().enumerate () {
      if 1 == pos {
        command = token.replace ("\"", "");
      }
    }
    return execute_process (command);
  }
  // Hey! I'm not soooo stupid. Give me something to do or don't call me!
  return to_string("");
}

fn execute_process (command : String) -> String {
  let mut cmd = Command::new ("");

  let cmd_tokens : Vec<&str> = command.split(" ").collect();
  for (pos, token) in cmd_tokens.iter().enumerate() {
    if 0 == pos {
      cmd = Command::new (token);
    }
    else {
      if 0 < token.len() {
        cmd.arg (token);
      }
    }
  }
  //DEBUG println! ("{:#?}", cmd);
      
  let cmd_result = cmd.output().expect ("404 program not found");
  let cmd_result_as_string = String::from_utf8_lossy (cmd_result.stdout.as_slice());
  return format! ("{}",cmd_result_as_string);
}

// Come on, let me do this work for you!
fn do_ssi_command (ssi_command : String) -> String {
  // incomming #exec "cmd" => split it do ist
  let tokens: Vec<&str> = ssi_command.split(" ").collect();

  let mut exec_result = "".to_string();
  for (pos, token) in tokens.iter().enumerate () {
    if 0 == pos {
      //println! ("command: {}",token);
      match token {
        &"#exec" => {
          exec_result = do_ssi_exec (&tokens);
          //DEBUG println! ("Method return: {}", exec_result);
          return exec_result;
        },
        _ => {
          // I'm lost in translate. Do you really know what you mean?
        }
      }
    }
  }  
// \n\r
  return to_string (&exec_result);
}

fn to_string (src : &str) -> String {
  return format! ("{}",src);
}

// Help me! I'm lost in EOF
