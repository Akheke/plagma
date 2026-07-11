//use base64::{Engine as _, engine::general_purpose};
use std::fs;
use std::io::{self,  Write};
//use std::os::unix::fs::PermissionsExt;
use std::process;
//use crossterm::event::{read, Event};
use crate::Output;



use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use rand_core::{OsRng};
use std::path::Path;
use x25519_dalek::{StaticSecret, PublicKey};



pub fn read_chunks(path: &std::path::PathBuf, is_quiet: bool) -> Result<Vec<u8>, ()> {
    
    let mut log: String = String::from("reading file...");
    output_log(&mut log, "start", is_quiet);
    /*
    let mut file = fs::File::open(path)?;
    let mut buf = [0u8; 1024];
    let mut result: Vec<u8> = Vec::new();
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        result.extend_from_slice(&buf[..n]);
    }
    Ok(result)
    */
    match fs::read(path) {
        Ok(s) => {
            output_log(&mut log, "success", is_quiet);
            Ok(s)
        },
        Err(e) => {
            eprintln!("failed to read file: {}",e);
            output_log(&mut log, "failed", is_quiet);
            process::exit(1);
        },
    }
}

pub fn handle_output(
    output: Output,
    output_path: Option<std::path::PathBuf>,
    result: String,
    quiet: bool,
    force: bool,
) {
    match output {
        Output::Std => {
            if output_path.is_some() && !quiet {
                println!("can't specify an output path when output=std.");
            }
            println!("===== RESULT =====\n{}", result);
        }

        Output::File => {
            let mut log = String::from("writing result for file...");
            output_log(&mut log, "start", quiet);

            let path = output_path
                .as_ref()
                .expect("output-path is required when output=file");

            if let Err(e) = fs::write(path, result) {
                eprintln!("failed to output to file: {}", e);
                output_log(&mut log, "failed", quiet);
                if !force {
                    process::exit(1);
                }
            } else {
                output_log(&mut log, "success", quiet);
            }
        }
    }
}

pub fn get_target(path: &Option<std::path::PathBuf>, string: &String, is_quiet: bool) -> Vec<u8> {
    if !string.is_empty() {
        return string.as_bytes().to_vec();
    }

    let path = path.as_ref().expect("target-path is required when no target string is provided");
    read_chunks(path, is_quiet).unwrap()
}

pub fn judge_mode(encrypt: bool, decode: bool, is_forced: bool) -> bool {
    let mode = match (encrypt, decode) {
    (true, false) => true,
    (false, true) => false,
    (false, false) => false, // デフォルト
    (true, true) => {
        eprintln!("An unexpected error occurred when selecting the mode");
        if !is_forced {
            process::exit(1);
        }
        false
    },
};
mode

}

pub fn output_log(log: &mut String, result: &str, is_quiet: bool) {
    if is_quiet {
        return;
    }
    match result {
        "start" => {
            print!("[INFO] {}", log);
        }
        "success" => {
            println!(" : \x1b[32mSUCCESS\x1b[0m");
        }
        "failed" => {
            print!(" : \x1b[31mFAILED\x1b[0m");
        }
        _ => {
            print!(" : \x1b[33mUNKNOWN\x1b[0m");
        }
    }
}

pub fn read_user_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}


pub fn find_order_files(
    dir: &std::path::Path,
    ext: &str,
    is_forced: bool,
    is_quiet: bool,
) -> io::Result<Vec<std::path::PathBuf>> {
    let mut files = Vec::new();

    let mut log: String = String::from("reading order file...");
    output_log(&mut log, "start", is_quiet);
    let entries = fs::read_dir(dir)?;

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => {
                if is_forced {
                    continue;
                } else {
                    output_log(&mut log, "failed", is_quiet);
                    eprintln!("Error:failed to read order file.");
                    process::exit(1)
                }
            }
        };

        let path = entry.path();

        if path.extension().and_then(|e| e.to_str()) == Some(ext) {
            files.push(path);
        }
    }

    output_log(&mut log, "success", is_quiet);

    Ok(files)
}

/* 
fn register_key(is_forced: bool,is_quiet: bool,register:bool,plugin_path: &std::path::Path) {
    let mut log = String::from("making keys...");
    output_log(&mut log, "start", is_quiet);
        let output = process::Command::new(&plugin_path)
            .arg("boo")
            .arg("false")
            .arg(is_forced.to_string())
            .arg(is_quiet.to_string())
            .arg("foo")
            .arg(register.to_string())
            .output()
            .expect("failed to execute process");

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if output.status.success() {
            output_log(&mut log, "success", is_quiet);
            println!("{}",stdout.to_string());
        } else {
            output_log(&mut log, "failed", is_quiet);
            eprintln!("failed to make key ");
            eprintln!("{}", stderr);

                process::exit(1);
            }
        }
*/

pub fn execute_process(
     plugin_paths: Vec<std::path::PathBuf>,
    code: &mut String,
    is_forced: bool,
    is_quiet: bool,
    encrypt: bool,
    decode: bool,
)->Result<(), ()> {
    for plugin_path in plugin_paths {
        if !plugin_path.exists() {
            eprintln!("Error: {:?} does not exist.", plugin_path);
            if !is_forced {
                process::exit(1);
            }
            continue;
        }
        if !is_executable(&plugin_path) {
            if !is_forced {
                eprintln!(
                    "[confirmation] An executable file was found.Would you like to continue?(Y/n)"
                );
                let mut is_checked: bool = false;
                while !is_checked {
                    let mut input = String::new();
                    match std::io::stdin().read_line(&mut input) {
                        Ok(_) => match input.trim() {
                            "Y" | "y" => {
                                is_checked = true;
                            }
                            "N" | "n" => {
                                process::exit(0);
                            }
                            other => {
                                eprintln!("Error: invalid input: {}.", other);
                            }
                        },
                        Err(e) => {
                            eprintln!("Error: failed to get input.\n{}", e)
                        }
                    }
                }
            } else {
                continue;
            }
        }

        let mut log: String = format!("executing process... path={}\n", plugin_path.display());
        output_log(&mut log, "start", is_quiet);
        let output = process::Command::new(&plugin_path)
            .arg(&*code)
            .arg(judge_mode(encrypt, decode, is_forced).to_string())
            .arg(is_forced.to_string())
            .arg(is_quiet.to_string())
            .output()
            .expect("failed to execute process");

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if output.status.success() {
            output_log(&mut log, "success", is_quiet);
            *code = stdout.to_string();
        } else {
            output_log(&mut log, "failed", is_quiet);
            eprintln!("failed to execute process: {}", plugin_path.display());
            eprintln!("{}", stderr);
            if !is_forced {
                process::exit(1);
            }
        }
    }

    Ok(())
}

pub fn cryptography(
    path: &std::path::Path,
    is_forced: bool,
    is_quiet: bool,
    target: Vec<u8>,
    encrypt: bool,
    decode: bool,
) -> String {
    if !is_quiet {
        print!("running process...");
    }

    let mut getting_target_log: String = String::from("retrieving target data...");
    output_log(&mut getting_target_log, "start", is_quiet);
    let mut code: String = String::from_utf8(target).unwrap();
    output_log(&mut getting_target_log, "success", is_quiet);
    let order_files = match find_order_files(path, "order", is_forced, is_quiet) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error: failed to read order files: {}", e);
            process::exit(1);
        }
    };

    match order_files.len() {
        0 => {
            eprintln!("Error: no order file found in {:?}", path);
            process::exit(1);
        }
        1 => {}
        _ => {
            eprintln!("Error: multiple order files found:");
            for f in &order_files {
                eprintln!("  {:?}", f);
            }
            if !is_forced {
                process::exit(1);
            }
        }
    }

    let order_file = &order_files[0];
    let content = match fs::read_to_string(order_file) {
        Ok(c) => c,
        Err(e) => {
            eprintln!(
                "failed to read plugin file.\n file'name:{:?}\nerror:{}",
                order_file, e
            );
            process::exit(1);
        }
    };
    let plugin_paths = content
        .split(";")
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|name| {
            let mut p = path.to_path_buf();
            p.push(name);
            p
        })
        .collect::<Vec<std::path::PathBuf>>();

        match execute_process(plugin_paths, &mut code, is_forced, is_quiet, encrypt, decode) {
            Ok(_) => {}
            Err(_) => {
                eprintln!("Error: failed to execute plugin process");
                if !is_forced {
                    process::exit(1);
                }
            }
        }

    code
}

pub fn create_key(is_forced: bool,
    is_quiet: bool,
    my_secret_path:&Path,
    my_pub_path:&Path
) {
    let _ = is_forced;
    let mut log_message = String::from("generating the key bytes...");
    output_log(&mut log_message, "start", is_quiet);

    let mut secret_b64 = String::new();
    let mut public_b64 = String::new();
    generate_keys(&mut secret_b64, &mut public_b64);

    output_log(&mut log_message, "success", is_quiet);


    let mut log_message = String::from("formatting the key bytes");
    output_log(&mut log_message, "start", is_quiet);


    let secret_bytes = STANDARD.decode(&secret_b64).unwrap();
    let public_bytes = STANDARD.decode(&public_b64).unwrap();

    let sec_arr:[u8; 32] = secret_bytes.try_into().unwrap();
    let pub_arr:[u8; 32] = public_bytes.try_into().unwrap();

    let sec_res = StaticSecret::from(sec_arr);
    let pub_res = PublicKey::from(pub_arr);


    output_log(&mut log_message, "success", is_quiet);


    //ここのエラー処理を作る（課題）
    let mut log_message = String::from("save the keys...");
    output_log(&mut log_message, "success", is_quiet);
    save_secret(&my_secret_path, &sec_res).expect("failed to save secret key");
    save_public(&my_pub_path, &pub_res).expect("failed to save public key");
    output_log(&mut log_message, "success", is_quiet);
        }


/*
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

#[cfg(unix)]
fn is_executable(path: &std::path::Path) -> bool {
    match path.metadata() {
        Ok(meta) => meta.permissions().mode() & 0o111 != 0,
        Err(_) => false,
    }
}
    */

#[cfg(windows)]
fn is_executable(path: &std::path::Path) -> bool {
    match path.extension().and_then(|s| s.to_str()) {
        Some(ext) => matches!(ext.to_ascii_lowercase().as_str(), "exe" | "bat" | "cmd" | "key"),
        None => false,
    }
}




// ----------------------------
// 鍵生成
// ----------------------------
pub fn generate_keys(secret: &mut String, public: &mut String) {
    let my_secret = StaticSecret::new(OsRng);
    let my_public = PublicKey::from(&my_secret);

    *secret = STANDARD.encode(my_secret.to_bytes());
    *public = STANDARD.encode(my_public.as_bytes());
}

// ----------------------------
// 秘密鍵の保存・読み込み
// ----------------------------
pub fn save_secret(path: &Path, secret: &StaticSecret) -> std::io::Result<()> {
    let encoded = STANDARD.encode(secret.to_bytes());
    fs::write(path, encoded)?;
    Ok(())
}

// ----------------------------
// 公開鍵の保存・読み込み
// ----------------------------
pub fn save_public(path: &Path, pubkey: &PublicKey) -> std::io::Result<()> {
    let encoded = STANDARD.encode(pubkey.as_bytes());
    fs::write(path, encoded)?;
    Ok(())
}