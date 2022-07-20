
use crate::subs::runnable::RunnableSubcommand;

use solar_ha_processing::{
    path,
    ser,
    lunar, 
    solar,
    vprintln,
    parallacticangle,
    enums::Target
};

use sciimg::{
    quality
};

use std::process;

#[derive(clap::Args)]
#[clap(author, version, about = "Print frame details", long_about = None)]
pub struct FrameStats {
    #[clap(long, short, help = "Input images", multiple_values(true))]
    input_files: Vec<String>,

    #[clap(long, short='T', help = "Target (Moon, Sun)")]
    target: Option<String>,

    #[clap(long, short, help = "Observer latitude", allow_hyphen_values(true))]
    latitude: f32,

    #[clap(long, short='L', help = "Observer longitude", allow_hyphen_values(true))]
    longitude: f32,
}   


impl RunnableSubcommand for FrameStats {
    fn run(&self) {

        let target = match &self.target {
            Some(t) => {
                match Target::from(&t) {
                    Some(t) => t,
                    None => {
                        eprintln!("Error: Unrecognized target value: {}", t);
                        process::exit(1);
                    }
                }
            },
            None => Target::Sun
        };


        println!("{:11} {:26} {:8}    {:9}    {:5} {:5}", "Frame Num:", "Date/Time:", "Sigma:", "Rotation:", "Min DN:", "Max DN:");

        self.input_files.iter().for_each(|sf| {

            if ! path::file_exists(sf) {
                panic!("File not found: {}", sf);
            }
    
            let ser_file = ser::SerFile::load_ser(sf).expect("Unable to load SER file");
            for i in 0..ser_file.frame_count {
    
                let frame_buffer = ser_file.get_frame(i).unwrap();
                let (alt, az) = match target {
                    Target::Moon => {
                        vprintln!("Calculating position for Moon");
                        lunar::position_from_lat_lon_and_time(self.latitude as f64, self.longitude as f64, &frame_buffer.timestamp)
                    },
                    Target::Sun => {
                        vprintln!("Calculating position for Sun");
                        solar::position_from_lat_lon_and_time(self.latitude as f64, self.longitude as f64, &frame_buffer.timestamp)
                    }
                };
        
                let rotation = parallacticangle::from_lat_azimuth_altitude(self.latitude as f64, az, alt);
                let (min, max) = frame_buffer.buffer.get_min_max_all_channel();
    
                let qual = quality::get_quality_estimation(&frame_buffer.buffer);
    
                println!("{:>10}  {}-{:02}-{:02} {:02}:{:02}:{:02}.{:04}   {:.4} {:>10.4} {:>7}    {:>7}", i, 
                                                        frame_buffer.timestamp.year, 
                                                        frame_buffer.timestamp.month, 
                                                        frame_buffer.timestamp.day,
                                                        frame_buffer.timestamp.hour,
                                                        frame_buffer.timestamp.minute,
                                                        frame_buffer.timestamp.second,
                                                        frame_buffer.timestamp.microsecond / 100,
                                                        qual,
                                                        rotation,
                                                        min, 
                                                        max);
    
            }

        });
        
    }
}