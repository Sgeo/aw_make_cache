#![allow(non_upper_case_globals)]

extern crate libloading as lib;

use std::os::raw::c_char;

use std::ffi::CString;

use std::fmt;
use std::error;


lazy_static! {
    static ref CT: lib::Library = lib::Library::new("ctreestd.dll").unwrap();
    static ref InitCTree: lib::Symbol<'static, unsafe extern "C" fn(i16, i16, i16) -> i16> = unsafe { CT.get(b"_INTREE\0").unwrap() };
    static ref AvailableFileNbr: lib::Symbol<'static, unsafe extern "C" fn(i16) -> i16> = unsafe { CT.get(b"_AVLFILNUM\0").unwrap() };
    static ref OpenCtFile: lib::Symbol<'static, unsafe extern "C" fn(i16, *const c_char, i16) -> i16> = unsafe { CT.get(b"_OPNFIL\0").unwrap() };
    static ref CloseCtFile: lib::Symbol<'static, unsafe extern "C" fn(i16, i16) -> i16> = unsafe { CT.get(b"_CLSFIL\0").unwrap() };
    static ref AddKey: lib::Symbol<'static, unsafe extern "C" fn(i16, *const u8, i32, i16) -> i16> = unsafe { CT.get(b"_ADDKEY\0").unwrap() };
    static ref NewVData: lib::Symbol<'static, unsafe extern "C" fn(i16, i32) -> i32> = unsafe { CT.get(b"_NEWVREC\0").unwrap() };
    static ref WriteVData: lib::Symbol<'static, unsafe extern "C" fn(i16, i32, *const u8, i32) -> i16> = unsafe { CT.get(b"_WRTVREC\0").unwrap() };
}

#[derive(Copy, Clone, Debug)]
pub struct Error(i16);

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "C-Tree error: {}", self.0)
    }
}

impl error::Error for Error {
    fn description(&self) -> &'static str {
        "C-Tree error"
    }
}

fn error(errcode: i16) -> Result<(), Error> {
    if errcode == 0 {
        Ok(())
    } else {
        Err(Error(errcode))
    }
}

pub fn init() -> Result<(), Error> {
    error(unsafe {
        InitCTree(3, 2, 32)
    })
}

#[derive(Copy, Clone, Debug)]
pub struct DatAddr(i32);

#[derive(Debug)]
pub struct DatFile(i16);
#[derive(Debug)]
pub struct IdxFile(i16);

impl DatFile {
    pub fn open<S: Into<Vec<u8>>>(filename: S) -> Result<Self, Error> {
        let filenum = unsafe { AvailableFileNbr(1) };
        if filenum == -1 {
            return Err(Error(-1));
        }
        let filename = CString::new(filename).unwrap();
        let result = unsafe { OpenCtFile(filenum, filename.as_ptr(), 0) };
        error(result).map(|_| DatFile(filenum))
    }
    
    fn new_v_data(&self, len: i32) -> Result<DatAddr, Error> {
        let result = unsafe {
            NewVData(self.0, len)
        };
        if result == 0 {
            Err(Error(0)) // Looks weird
        } else {
            Ok(DatAddr(result))
        }
    }
    
    fn write_v_data(&self, addr: &DatAddr, data: &[u8]) -> Result<(), Error> {
        error(unsafe {
            WriteVData(self.0, addr.0, data.as_ptr(), data.len() as i32)
        })
    }
    
    
}

impl Drop for DatFile {
    fn drop(&mut self) {
        error(unsafe {
            CloseCtFile(self.0, 0)
        }).expect("Unable to correctly close a DatFile!")
    }
}

impl IdxFile {
    pub fn open<S: Into<Vec<u8>>>(filename: S) -> Result<Self, Error> {
        let filenum = unsafe { AvailableFileNbr(1) };
        if filenum == -1 {
            return Err(Error(-1));
        }
        let filename = CString::new(filename).unwrap();
        let result = unsafe { OpenCtFile(filenum, filename.as_ptr(), 0) };
        error(result).map(|_| IdxFile(filenum))
    }
    
    fn add_key(&self, key: &[u8], dataddr: &DatAddr) -> Result<(), Error> {
        error(unsafe {
            AddKey(self.0, key.as_ptr(), dataddr.0, 0)
        })
    }
}

impl Drop for IdxFile {
    fn drop(&mut self) {
        error(unsafe {
            CloseCtFile(self.0, 0)
        }).expect("Unable to correctly close an IdxFile!")
    }
}

pub fn insert(idx: &IdxFile, dat: &DatFile, key: &[u8], data: &[u8]) -> Result<(), Error> {
    let addr = dat.new_v_data(data.len() as i32)?;
    dat.write_v_data(&addr, data)?;
    idx.add_key(key, &addr)?;
    Ok(())
}

