use failure;
use aw::Object;
use regex::Regex;

use std::io::Write;

pub struct Teleports {
    regions: Vec<((i16, i16), (i16, i16))>
}

fn coord_to_num<S: AsRef<str>>(coord: S) -> Result<i16, failure::Error> {
    use std::str::FromStr;

    let coord = coord.as_ref();
    let (digits, indicator) = coord.split_at(coord.len() - 1);
    let indicator = indicator.to_uppercase();
    let floating = f32::from_str(digits)?;
    let integer = floating as i16;
    if indicator == "N" || indicator == "W" {
        return Ok(integer);
    } else if indicator == "S" || indicator == "E" {
        return Ok(-integer);
    } else {
        bail!("Unable to process coordinate in teleport file!");
    }
}

fn bounds(coord: i16, radius: i16) -> (i16, i16) {
    (coord.saturating_sub(radius), coord.saturating_add(radius))
}

impl Teleports {
    pub fn from_file<P: AsRef<::std::path::Path>>(path: P, radius: i16) -> Result<Self, failure::Error> {
        use std::fs::File;
        use std::io::prelude::*;
        use std::io::BufReader;
        
        let mut this = Teleports {
            regions: Vec::new()
        };
        
        let file = File::open(path)?;
        let buffer = BufReader::new(file);
        
        for line in buffer.lines() {
            let mut line = line?;
            let mut coords = line.split(':').next().expect("Unable to split on : in teleport fille!");
            let mut data = coords.split(' ');
            let _world = data.next();
            let ns = data.next();
            let ew = data.next();
            ensure!(ns.is_some() && ew.is_some(), "Unable to process line in teleport file!");
            let z = coord_to_num(ns.unwrap())?;
            let x = coord_to_num(ew.unwrap())?;
            this.regions.push((bounds(x, radius), bounds(z, radius)));
        }
        
        Ok(this)
    }

    pub fn contains(&self, object: &Object) -> bool {
        let location = object.location();
        for ((min_x, max_x), (min_z, max_z)) in &self.regions {
            if min_x <= &location.cell_x && &location.cell_x <= max_x && min_z <= &location.cell_z && &location.cell_z <= max_z {
                return true;
            }
        }
        return false;
    }
}

pub struct TeleportAppender {
    file: ::std::fs::File,
    world: String
}

impl TeleportAppender {
    pub fn from_file<P: AsRef<::std::path::Path>, S: AsRef<str>>(path: P, world: S) -> Result<Self, failure::Error> {
        use std::fs::OpenOptions;
        let file = OpenOptions::new().create(true).append(true).open(path)?;
        Ok(TeleportAppender {
            file: file,
            world: world.as_ref().to_uppercase()
        })
    }
    
    pub fn check_to_append(&mut self, object: &Object) -> Result<(), failure::Error> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"(?i)\W(teleportx?|warp)\W+((?P<world>\w+)\W+)?(?P<ns>[0-9.]+(n|s))\W+(?P<ew>[0-9.]+(e|w))").unwrap();
        }
        for capture in RE.captures_iter(&object.action) {
            if let Some(world) = capture.name("world") {
                if world.as_str().to_uppercase() != self.world {
                    continue;
                }
            }
            let ns = capture.name("ns").expect("Couldn't find coords in teleport").as_str();
            let ew = capture.name("ew").expect("Couldn't find coords in teleport").as_str();
            writeln!(&mut self.file, "{} {} {}: ZZZFound", &self.world, ns, ew)?;
        }
        Ok(())
    }
}
