mod turret;
mod error;

use error::ScopeError;
use log::debug;
use serialport::{self, SerialPort};
use turret::ScopeTurret;
use std::time::Duration;
use std::error::Error;
use std::thread::sleep;
use std::time::Instant;

#[derive(Debug)]
pub struct Scope {
    pub scope_port: Box<dyn SerialPort>,
}

impl Scope {
    /// Communication timout
    const TIMEOUT: Duration = Duration::from_millis(2000);

    /// Size of focus steps in micrometers
    const STEP_SIZE: f64 = 0.050;

    /// Baud rate of the serial connection
    const BAUD_RATE: u32 = 9600;

    pub fn new(scope_port: &str) -> Result<Self, ScopeError> {
        let scope_port = serialport::new(scope_port, Self::BAUD_RATE)
            .timeout(Self::TIMEOUT)
            .open()?;

        Ok(Scope {
            scope_port,
        })
    }

    /// Query the scope, sending a packet and getting some response
    pub fn query_scope(
        &mut self,
        query: &str,
        expect_response: bool
    ) -> Result<Option<Vec<u8>>, ScopeError> {
        let mut query_bytes = query.as_bytes().to_vec();
        query_bytes.push(b'\r');
        let mut read_buffer: Vec<u8> = vec![];

        self.scope_port.clear(serialport::ClearBuffer::All).unwrap();

        // Send the specified query to the scope
        self.scope_port.write_all(&query_bytes)?;
        self.scope_port.flush()?;

        // Exit early if no response expected
        if !expect_response {
            return Ok(None)
        }

        // Read bytes from the port until there are none left
        let mut elapsed = Instant::now();
        loop {
            let avail =  self.scope_port.bytes_to_read().unwrap();
            if avail == 0 {
                sleep(Duration::from_millis(5));
                if elapsed.elapsed() >= Self::TIMEOUT {
                    break;
                }
                continue;
            }

            // Reset the timeout if able to read
            elapsed = Instant::now();
            let mut chunk: Vec<u8> = vec![0u8; avail as usize];
            self.scope_port.read_exact(&mut chunk).unwrap();
            read_buffer.append(&mut chunk.clone());
            if chunk.contains(&13) {
                break
            }
        }

        Ok(Some(read_buffer))
    }

    /// Validate the query and get data from it
    fn validate(
        &self,
        query: &str,
        result: &[u8]
    ) -> Result<Vec<u8>, ScopeError> {
        if result.len() < 2 {
            return Err(ScopeError::EmptyResponse)
        }

        let mut query_check = query.as_bytes()[0..2].to_vec();
        query_check.reverse();

        match query_check == result[0..2].to_vec() {
            true => Ok(result[2..result.len() - 1].to_vec()),
            false => Err(
                ScopeError::QueryValidation(
                    String::from_utf8(query_check.to_vec()).unwrap_or_default(),
                    String::from_utf8(result[0..2].to_vec()).unwrap_or_default()
                )
            )
        }
    }

    fn query_validate(&mut self, query: &str, expect_response: bool) -> Result<Option<Vec<u8>>, ScopeError> {
        let res = self.query_scope(&query, expect_response)?;

        match expect_response {
            true => self.validate(&query, &res.unwrap()).map(|v| Some(v)),
            false => Ok(None)
        }
    }

    pub fn firmware_version(&mut self) -> Result<(String, String), ScopeError> {
        let query = format!("HPTv0");
        let result = self.query_validate(&query, true)?.unwrap();

        //println!("{}", String::from_utf8_lossy(&result));
        let result = String::from_utf8_lossy(&result);

        let version_strings = result
            .split("_")
            .collect::<Vec<&str>>();

        if version_strings.len() != 2 {
            return Err(ScopeError::InvalidResponse)
        }

        debug!("{}, {}", version_strings[0], version_strings[1]);

        Ok((version_strings[0].to_string(), version_strings[1].to_string()))
    }

    /// Gets the position of a "Turret"
    pub fn turret_pos(
        &mut self,
        turret: ScopeTurret,
    ) -> Result<u8, ScopeError> {
        let query = format!("HPCr{},1", turret as u8);

        let response = self.query_validate(&query, true)?.unwrap();
        Ok(u8::from_str_radix(&String::from_utf8(response)?, 10)?)
    }

    /// Sets the position of a "Turret"
    pub fn set_turret_pos(
        &mut self,
        turret: ScopeTurret,
        position: u8
    ) -> Result<(), ScopeError> {
        if position > turret.positions() {
            return Err(ScopeError::OutOfRange(
                position as u64,
                turret.positions() as u64
            ));
        }

        let query = format!("HPCR{},{}", turret as u8, position);
        self.query_validate(&query, false)?;

        Ok(())
    }

    /// Gets the light diaphragm aperture
    pub fn light_diaphragm_aperture(&mut self) -> Result<u8, ScopeError> {
        let query = "HPCs4,1";
        let response = self.query_validate(query, true)?.unwrap();
        let res_string = String::from_utf8(response).unwrap();

        Ok(res_string.parse().unwrap())
    }

    /// Sets the light diaphragm aperture
    pub fn set_light_diaphragm_aperture(&mut self, position: u8) -> Result<(), Box<dyn Error>> {
        let query = format!("HPCS4,{}", position);

        self.query_validate(&query, false)?;

        Ok(())
    }

    /// Gets the focus distance (Z) in steps
    pub fn focus_distance(&mut self) -> Result<i64, Box<dyn Error>> {
        let query = "FPZp";

        let result = self.query_validate(query, true)?.unwrap();
        let mut result_24 = hex::decode(result).unwrap().to_vec();
        result_24.reverse();
        result_24.push(0);
        result_24.reverse();

        let final_number = zeiss_to_i64(i32::from_be_bytes(result_24[0..4].try_into().unwrap()) as i64);
        Ok(final_number)
    }

    /// Sets the focus distance (Z) in steps
    pub fn set_focus_distance(&mut self, distance: i64) -> Result<(), Box<dyn Error>> {
        let output_num = format!("{:06X?}", i64_to_zeiss(distance));

        let mut query = "FPZT".to_string();
        query.push_str(&output_num);

        println!("{output_num}\n{query}");

        self.query_validate(&query, false).unwrap();

        Ok(())
    }

    pub fn focus_limit_upper(&mut self) -> Result<i64, Box<dyn Error>> {
        let query = "FPZu";
        let response = self.query_validate(query, true)?.unwrap();
        let mut result_24 = hex::decode(response).unwrap().to_vec();
        result_24.reverse();
        result_24.push(0);
        result_24.reverse();
        let value = zeiss_to_i64(i32::from_be_bytes(result_24.try_into().unwrap()) as i64);

        Ok(value)
    }

    pub fn focus_limit_lower(&mut self) -> Result<i64, Box<dyn Error>> {
        let query = "FPZl";
        let response = self.query_validate(query, true)?.unwrap();
        let mut result_24 = hex::decode(response).unwrap().to_vec();
        result_24.reverse();
        result_24.push(0);
        result_24.reverse();
        let value = zeiss_to_i64(i32::from_be_bytes(result_24[0..4].try_into().unwrap()) as i64);

        Ok(value)
    }

    /// Gets the focus distance (Z) in micrometers (μm)
    pub fn focus_distance_um(&mut self) -> Result<f64, Box<dyn Error>> {
        Ok(Self::STEP_SIZE * self.focus_distance()? as f64)
    }

    /// Sets the focus distance (Z) in micrometers (μm)
    pub fn set_focus_distance_um(&mut self, distance: f64) -> Result<(), Box<dyn Error>> {
        let new_distance = distance / Self::STEP_SIZE;

        self.set_focus_distance(new_distance as i64)
    }
}

pub fn zeiss_to_i64(input_num: i64) -> i64 {
    let mut result = input_num;
    if input_num >= 0x00800000 {
        result = -(0x00FFFFFF - input_num);
    }

    result
}

pub fn i64_to_zeiss(input_num: i64) -> i64 {
    let mut result = input_num;
    if input_num < 0 {
        result = 0x00FFFFFF - -(input_num | 0x00F00000);
    }

    result
}
