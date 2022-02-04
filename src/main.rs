
extern crate exitcode;
use std::process;
use std::io::{Error, ErrorKind};

use std::env;
use std::str::FromStr;

use std::path::PathBuf;
use rand::Rng;

use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::{thread, time};
use rodio::{Decoder, Sink, OutputStream, OutputStreamHandle};

fn main()
{
	let args: Vec<String> = env::args().collect();

	if args.contains(&String::from("-h"))
	{
		println!("muzak-rs\n\t-f [fade in (s)] default: 5
		\n\t-m [min delay (min)] default: 5
		\n\t-x [max delay (min)] default: 10
		\n\t-l [loop music] default: false");
	}
	else
	{
		let fade_length = match parse_value_flag::<u32>(args.clone(), String::from("-f"))
		{
			Some(i) => i,
			None => 5
		};
		let min_delay: u64 = match parse_value_flag::<f32>(args.clone(), String::from("-m"))
		{
			Some(i) => (i * 60f32) as u64,
			None => 300
		};
		let max_delay: u64 = match parse_value_flag::<f32>(args.clone(), String::from("-x"))
		{
			Some(i) => (i * 60f32) as u64,
			None => 600
		};
		let loop_bool: bool = match parse_bool_flag(args.clone(), String::from("-l"))
		{
			Some(_) => true,
			None => false
		};

		match play_dir(fade_length, min_delay, max_delay, loop_bool)
		{
			Ok(()) => process::exit(exitcode::OK),
			Err(e) => 
			{
				eprintln!("Error: {}", e);
				process::exit(exitcode::DATAERR);
			}
		};
	}
}

fn parse_value_flag<T: FromStr>(args: Vec<String>, flag: String) -> Option<T>
{
	if args.contains(&flag)
	{
		let index = match args.iter().position(|a| a == &flag)
		{
			Some(i) => i + 1,
			None => return None
		};

		if args.len() > index
		{
			match args[index].parse::<T>()
			{
				Ok(i) => return Some(i),
				Err(_) => panic!("Could not parse arguments")
			};
		}
	}

	None
}

fn parse_bool_flag(args: Vec<String>, flag: String) -> Option<bool>
{
	if args.contains(&flag)
	{
		return Some(true);
	}
	
	None
}

fn play_dir(fade_length: u32, min_delay: u64, max_delay: u64, loop_bool: bool) -> Result<(), Error>
{
	let mut files = match get_files()
	{
		Some(v) => v,
		None => return Err(Error::from(ErrorKind::InvalidData))
	};

	let (_stream, handle) = OutputStream::try_default().unwrap();

	let mut last_song: usize = 0;
	let mut rng = rand::thread_rng();

	if loop_bool
	{
		loop
		{
			loop
			{
				let index = rng.gen_range(0..files.len());
				if files.len() > 2 && index != last_song
				{
					last_song = index;
					break;
				}
			}

			play_file(&handle, &files[last_song], fade_length, rng.gen_range(min_delay..max_delay))?;
		}
	}
	else
	{
		while files.len() > 0
		{
			let index = rng.gen_range(0..files.len());
			play_file(&handle, &files[index], fade_length, rng.gen_range(min_delay..max_delay))?;
			files.remove(index);
		}
	}

	Ok(())
}

fn play_file(handle: &OutputStreamHandle, file_path: &PathBuf, fade_length: u32, delay: u64) -> Result<(), Error>
{

	let sink = match Sink::try_new(&handle)
	{
		Ok(s) => s,
		Err(_) => return Err(Error::from(ErrorKind::Other))
	};

	sink.set_volume(0f32);

	let file = match File::open(file_path)
	{
		Ok(d) => d,
		Err(_) => return Err(Error::from(ErrorKind::InvalidData))
	};

	println!("Now Playing: {}", &file_path.file_stem().unwrap().to_str().unwrap());

	let decoded_file = match Decoder::new(BufReader::new(file))
	{
		Ok(d) => d,
		Err(_) => return Err(Error::from(ErrorKind::InvalidData))
	};

	sink.append(decoded_file);

	let sink = fade_in(sink, fade_length);

	sink.sleep_until_end();

	thread::sleep(time::Duration::from_secs(delay));

	Ok(())
}

fn get_files() -> Option<Vec<PathBuf>>
{
	let working_dir = match env::current_dir()
	{
		Ok(p) => p,
		Err(_) => return None
	};
	let files: Vec<PathBuf> = fs::read_dir(working_dir).unwrap().filter_map(|f|
	{
		let f = f.unwrap();
		let path = f.path();
		let path_extension = match path.extension()
		{
			Some(e) => e,
			None => return None
		};
		match path_extension.to_str().unwrap()
		{
			"flac" | "wav" | "mp3" => return Some(path),
			_ => return None
		}
	}).collect();

	if files.len() == 0
	{
		return None;
	}

	Some(files)
}

fn fade_in(sink: rodio::Sink, fade_length: u32) -> rodio::Sink
{
	let time_increment = 2;
	let steps = fade_length * 1000 / 2;
	let increment: f32 = 1f32 / (steps as f32);
	for i in 0..steps
	{
		sink.set_volume((i as f32) * increment + increment);
		thread::sleep(time::Duration::from_millis(time_increment));
	}

	sink
}
