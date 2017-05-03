use serde_json;
use sputnikvm;

use sputnikvm::{read_hex, Gas, M256, Address};
use sputnikvm::vm::{Machine, VectorMachine, Stack, PC};
use sputnikvm::blockchain::Block;
use sputnikvm::transaction::{Transaction, VectorTransaction};

use super::json_schema::{create_machine, test_machine};

use serde_json::{Value, Error};
use std::fs::File;
use std::path::Path;
use std::io::{BufReader, Write, stdout};
use std::str::FromStr;

use rustyline::error::ReadlineError;
use rustyline::Editor;

pub fn test_transaction(name: &str, v: &Value, debug: bool) {
    print!("Testing {} ... ", name);
    if debug {
        print!("\n");
    }
    stdout().flush();

    let mut machine = create_machine(v);
    let result = machine.fire();

    let out = v["out"].as_str();

    if out.is_some() {
        if result.is_ok() {
            if test_machine(v, &machine, debug) {
                println!("OK");
            } else {
                println!("Failed (result not match)");
            }
        } else {
            println!("Failed {:?}", result.err().unwrap());
        }
    } else {
        if result.is_err() {
            println!("OK");
        } else {
            println!("Failed");
        }
    }
}


pub fn debug_transaction(v: &Value) {
    let mut machine = create_machine(v);
    let mut rl = Editor::<()>::new();

    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(&line);
                match line.as_ref() {
                    "step" => {
                        if machine.pc().stopped() {
                            println!("Stopped");
                        } else {
                            println!("Running {:?} ... {:?}.", machine.pc().peek_opcode(),
                                     machine.step());
                        }
                    },
                    "fire" => {
                        let result = machine.fire();
                        println!("{:?}", result);
                    },
                    "fire debug" => {
                        while !machine.pc().stopped() {
                            println!("Running {:?} ...", machine.pc().peek_opcode());
                            let gas = machine.peek_cost().unwrap();
                            if gas < Gas::from(u64::max_value()) {
                                let gas: u64 = gas.into();
                                println!("Cost: {}", gas);
                            } else {
                                println!("Cost: 0x{:x}", gas);
                            }
                            for i in 0..machine.stack().size() {
                                println!("{}: {:x}", i, machine.stack().peek(i).unwrap());
                            }
                            println!("Result: {:?}", machine.step());
                            print!("\n");
                        }
                    },
                    "gas" => {
                        let gas = machine.peek_cost();
                        if gas.is_ok() {
                            println!("0x{:x}", gas.unwrap());
                        } else {
                            println!("{:?}", gas);
                        }
                    }
                    "out" => {
                        let ret = machine.return_values();
                        println!("{:?}", ret);
                    }
                    "print stack" => {
                        for i in 0..machine.stack().size() {
                            println!("{}: {:x}", i, machine.stack().peek(i).unwrap());
                        }
                    },
                    _ => {
                        println!("Unknown command.");
                    }
                }
            },
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break
            },
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break
            }
        }
    }
}