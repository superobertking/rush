/// @Author: robertking
/// @Date:   2017-03-13 00:45:22
/// @Last Modified by:   robertking
/// @Last Modified time: 2017-03-24 23:41:10


// Imported libraries

use std::io::prelude::*;
use std::io;
use std::env;
use std::path::{Path, PathBuf};
use std::process;
use std::process::{Child, Command, Stdio};
use std::fs::File;
use std::os::unix::io::{IntoRawFd, FromRawFd, AsRawFd};
extern crate libc;
use libc::{waitpid, WNOHANG, kill, SIGTERM};

// Constants
const PS1: &'static str = "$ ";
// const PS2: &'static str = "> ";

// Sturct Definitions

struct Cmd {
    cmd: String,
    argv: Vec<String>,
    handle: Option<Child>,
    is_builtin: bool,
}

#[derive(Debug)]
struct _ExtraInfo {
    repr_string: String,
    input_filename: Option<String>,
    output_filename: Option<String>,
    is_daemon: bool,
}

struct CmdLine {
    cmds: Vec<Cmd>,
    repr_string: String,
    input_filename: Option<String>,
    output_filename: Option<String>,
    is_daemon: bool,
    running: bool,
    newly_added: bool,
}

// Struct Implements

impl Cmd {
    fn new(cmd_str: String) -> Result<Option<(Self, _ExtraInfo)>, &'static str> {
        let mut new_cmd = Cmd {
            cmd: String::new(),
            argv: Vec::<String>::new(),
            handle: None,
            is_builtin: false,
        };
        let mut extrainfo = _ExtraInfo {
            repr_string: {
                let mut tokens: Vec<&str> = cmd_str.split_whitespace().collect();
                if let Some(&last_token) = tokens.last() {
                    if last_token == "&" {
                        tokens.pop();
                    }
                }
                tokens.join(" ")
            },
            input_filename: None,
            output_filename: None,
            is_daemon: false,
        };
        let argv_list: Vec<String> = cmd_str.split_whitespace().map(|x| x.to_string()).collect();
        let mut argv_iter = argv_list.into_iter();
        if let Some(cmd) = argv_iter.next() {
            match cmd.as_str() {
                "cd" | "exit" | "history" | "jobs" | "kill" | "pwd" => new_cmd.is_builtin = true,
                _ => {}
            }
            new_cmd.cmd = cmd;
        } else {
            return Ok(None);
        }
        while let Some(token) = argv_iter.next() {
            match token.as_str() {
                "&" => extrainfo.is_daemon = true,
                "<" | ">" => {
                    if let Some(filename) = argv_iter.next() {
                        if token == "<" {
                            extrainfo.input_filename = Some(filename);
                        } else {
                            extrainfo.output_filename = Some(filename);
                        }
                    } else {
                        return Err("No direction file name specified!");
                    }
                }
                _ => new_cmd.argv.push(token),
            }
        }
        Ok(Some((new_cmd, extrainfo)))
    }
}
// 'A'...'Z' | 'a'...'z' | '0'...'9' | '.' | '/' | '-' | '_' => {},

impl CmdLine {
    fn new(cmd_line: String) -> Result<Self, &'static str> {
        let mut new_cmdline = CmdLine {
            cmds: Vec::<Cmd>::new(),
            repr_string: String::new(),
            input_filename: None,
            output_filename: None,
            is_daemon: false,
            running: false,
            newly_added: true,
        };
        let cmd_vec: Vec<String> = cmd_line.split('|').map(|x| x.to_string()).collect();
        for i in 0..cmd_vec.len() {
            match Cmd::new(cmd_vec[i].to_string()) {
                Err(reason) => return Err(reason),
                Ok(ok_cmd) => {
                    if let Some((cmd, extrainfo)) = ok_cmd {
                        if new_cmdline.repr_string.len() != 0 {
                            new_cmdline.repr_string += " | ";
                        }
                        new_cmdline.repr_string += extrainfo.repr_string.as_str();
                        if let Some(_) = extrainfo.input_filename {
                            if i == 0 {
                                new_cmdline.input_filename = extrainfo.input_filename;
                            } else {
                                return Err("Input direction can only appear in the first \
                                              command!");
                            }
                        }
                        if let Some(_) = extrainfo.output_filename {
                            if i == cmd_vec.len() - 1 {
                                new_cmdline.output_filename = extrainfo.output_filename;
                            } else {
                                return Err("Output direction can only appear in the last command!");
                            }
                        }
                        if extrainfo.is_daemon {
                            if i == cmd_vec.len() - 1 {
                                new_cmdline.is_daemon = true;
                            } else {
                                return Err("Daemon character(&) can only appear in the last \
                                            command!");
                            }
                        }
                        new_cmdline.cmds.push(cmd);
                    }
                }
            }
        }
        Ok(new_cmdline)
    }
}

// Seperated Functions

fn print_prompt() {
    print!("{}", PS1);
    io::stdout().flush().ok().expect("Could not flush stdout!");
}

fn read_line() -> Option<String> {
    let mut cmd_line = String::new();
    let bytes_read = io::stdin().read_line(&mut cmd_line).expect("Could not read a line!");
    if bytes_read == 0 {
        return None;
    }   // EOF
    Some(cmd_line.trim_right_matches("\r\n").trim_right_matches("\n").to_owned())
}

fn chdir(path: &String) {
    env::set_current_dir(&Path::new(path.as_str())).unwrap(); // unsafe { libc::chdir() }
}

fn get_pwd() -> PathBuf {
    env::current_dir().unwrap() // cdir.to_str()
}

fn try_wait(pid: i32) -> bool {
    unsafe {
        let mut x = 0;
        match waitpid(pid, &mut x, WNOHANG) {
            0 => return true,
            _ => return false,
        }
    }
}

fn kill_pid(pid: i32) {
    unsafe {
        kill(pid as i32, SIGTERM);
    }
}

fn show_jobs(jobs: &mut Vec<CmdLine>) -> String {
    let mut ans = String::new();
    for job in jobs {
        if !job.newly_added {
            if job.running {
                let mut flag = false;
                for cmd in &job.cmds {
                    if let Some(ref handle) = cmd.handle {
                        if try_wait(handle.id() as i32) {
                            flag = true;
                            break;
                        }
                    }
                }
                if flag {
                    ans += format!("{}\n", &job.repr_string).as_str();
                } else {
                    job.running = false;
                }
            }
        }
    }
    ans
}

fn builtin_run(cmd: &String,
               argv: &Vec<String>,
               stdout: &mut String,
               history: &String,
               jobs: &mut Vec<CmdLine>) {
    *stdout = String::new();
    match cmd.as_str() {
        "cd" => {
            let tmp_str = env::home_dir().unwrap().to_str().unwrap().to_string();
            chdir({
                if argv.len() == 0 { &tmp_str } else { &argv[0] }
            });
        }
        "exit" => process::exit(0),
        "history" => *stdout = history.clone(),
        "jobs" => *stdout = show_jobs(jobs),
        "kill" => kill_pid(argv[0].parse::<i32>().unwrap()),
        "pwd" => *stdout = get_pwd().to_str().unwrap().to_string() + "\n",
        _ => {}
    }
}

fn start(clu: &mut CmdLine, history: &String, jobs: &mut Vec<CmdLine>) {
    let is_daemon = clu.is_daemon;
    clu.running = true;
    let len = clu.cmds.len();
    let output_filename = clu.output_filename.clone();
    let input_filename = clu.input_filename.clone();
    let mut builtin_output = String::new();
    let mut last_is_builtin = false;
    for i in 0..len {
        if clu.cmds[i].is_builtin {
            builtin_run(&clu.cmds[i].cmd, &clu.cmds[i].argv, &mut builtin_output, history, jobs);
            if !builtin_output.is_empty() {
                // len would not be 0
                if len == 1 {
                    print!("{}", &builtin_output);
                    io::stdout().flush().ok().expect("Could not flush stdout!");
                } else if len > 1 {
                    // i == 0 does nothing, i in the middle does nothing, i at the last:
                    if i == len - 1 {
                        if let Some(ref filename) = output_filename {
                            File::create(Path::new(filename.as_str())).unwrap()
                                .write_all(builtin_output.as_bytes()).unwrap();
                        } else {
                            print!("{}", &builtin_output);
                            io::stdout().flush().ok().expect("Could not flush stdout!");
                        }
                    }
                }
            }
            last_is_builtin = true;
        } else {
            let mut stdin = Stdio::inherit();
            let mut stdout = Stdio::inherit();
            if len == 1 {
                if let Some(ref filename) = input_filename {
                    stdin = unsafe {
                        Stdio::from_raw_fd(File::open(Path::new(filename.as_str()))
                            .unwrap().into_raw_fd())
                    };
                }
                if let Some(ref filename) = output_filename {
                    stdout = unsafe {
                        Stdio::from_raw_fd(File::create(Path::new(filename.as_str()))
                            .unwrap().into_raw_fd())
                    };
                }
            } else if len > 1 {
                if i == 0 {
                    if let Some(ref filename) = input_filename {
                        stdin = unsafe {
                            Stdio::from_raw_fd(File::open(Path::new(filename.as_str()))
                                .unwrap().into_raw_fd())
                        };
                    }
                    stdout = Stdio::piped();
                } else {
                    if last_is_builtin {
                        stdin = Stdio::piped();
                    } else {
                        stdin = unsafe {
                            Stdio::from_raw_fd(clu.cmds[i - 1].handle.as_mut().unwrap()
                                .stdout.as_mut().unwrap().as_raw_fd())
                        };
                    }
                    if i == len - 1 {
                        if let Some(ref filename) = output_filename {
                            stdout = unsafe {
                                Stdio::from_raw_fd(File::create(Path::new(filename.as_str()))
                                    .unwrap().into_raw_fd())
                            };
                        }
                    } else {
                        stdout = Stdio::piped();
                    }
                }
            }
            clu.cmds[i].handle = Some(Command::new(clu.cmds[i].cmd.as_str())
                .args(clu.cmds[i].argv.iter().map(|x| x.as_str()).collect::<Vec<&str>>().as_slice())
                .stdin(stdin).stdout(stdout).spawn().unwrap());
            if last_is_builtin {
                clu.cmds[i].handle.as_mut().unwrap().stdin.as_mut().unwrap()
                    .write_all(builtin_output.as_bytes()).unwrap();
            }
            last_is_builtin = false;
        }
    }
    clu.newly_added = false;
    if !is_daemon {
        for cmd in &mut clu.cmds {
            if let Some(ref mut handle) = cmd.handle {
                handle.wait().unwrap();
            }
        }
        clu.running = false;
    }
}

// EntryPoint

fn main() {
    // Static global variable uses unsafe to access, changed to local.
    let mut history = String::new();
    let mut jobs = Vec::<CmdLine>::new();
    let mut cmdline_cnt = 0;
    loop {
        print_prompt();
        if let Some(cmd_line_str) = read_line() {
            match CmdLine::new(cmd_line_str.clone()) {
                Ok(cmd_line) => {
                    let is_daemon = cmd_line.is_daemon;
                    let mut new_job = cmd_line;
                    start(&mut new_job, &history, &mut jobs);
                    if is_daemon {
                        jobs.push(new_job);
                    }
                    cmdline_cnt += 1;
                    history += format!("{:5}  {}\n", cmdline_cnt, cmd_line_str).as_str();
                }
                Err(reason) => println!("rush: {}", reason),
            }
        } else {
            break;
        }
    }
}
