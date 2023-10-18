use serialport::{self, SerialPort};
use std::time::Duration;
use std::error::Error;
use std::thread::sleep;
use std::time::Instant;
use hex;

/*
 * Useful things to know:
 *
 *
 *
 *
 *
 *
 *
 *
 *
 *
 */

const TIMEOUT: u128 = 200;

pub struct Scope {
    pub scope_port: Box<dyn SerialPort>,
    pub stage_port: Box<dyn SerialPort>,
}

#[derive(Debug)]
pub struct ScopeResponse {
    response: Vec<u8>,
    response_size: usize
}

#[derive(Debug, Clone, Copy)]
pub enum ScopeTurret {
    Reflector = 1,
    Objective = 2,
    DensityFilter1 = 3,
    DensityFilter2 = 4,
    Condenser = 5,
}

impl ScopeTurret {
    pub fn positions(self) -> u8 {
        match self {
            Self::Reflector => 0,
            Self::Objective => 6,
            Self::DensityFilter1 => 4,
            Self::DensityFilter2 => 4,
            Self::Condenser => 0,
        }
    }
}

impl Scope {
    const STEP_SIZE: f64 = 0.050;

    pub fn new(scope_port: &str, stage_port: &str) -> Result<Self, Box<dyn Error>> {
        let scope_port = serialport::new(scope_port, 9600)
            .timeout(Duration::from_millis(10))
            .open()?;

        let stage_port = serialport::new(stage_port, 9600)
            .timeout(Duration::from_millis(10))
            .open()?;

        Ok(Scope {
            scope_port,
            stage_port,
        })
    }

    pub fn query_scope(&mut self, query: &str) -> Result<ScopeResponse, Box<dyn Error>> {
        let query_bytes = query.as_bytes();
        let mut read_buffer: Vec<u8> = vec![];

        self.scope_port.clear(serialport::ClearBuffer::All).unwrap();

        // Send the specified query to the scope
        self.scope_port.write(query_bytes)?;

        // Read bytes from the port until there are none left
        let mut elapsed = Instant::now();
        loop {
            let avail =  self.scope_port.bytes_to_read().unwrap();
            if avail == 0 {
                sleep(Duration::from_millis(5));
                if elapsed.elapsed().as_millis() >= TIMEOUT {
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

        let total = read_buffer.len();

        Ok(ScopeResponse {
            response: read_buffer,
            response_size: total
        })
    }

    pub fn query_scope_print(&mut self, query: &str) -> Result<(), Box<dyn Error>> {
        let result = self.query_scope(query)?;
        println!("{:?}", result.response);
        println!("{}\r", String::from_utf8(result.response).unwrap());
        Ok(())
    }

    fn validate(
        &self,
        query: &str,
        result: &Vec<u8>
    ) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut query_check = query.as_bytes()[0..2].to_vec();
        query_check.reverse();

        match query_check == result[0..2].to_vec() {
            true => Ok(result[2.. result.len() - 1].to_vec()),
            false => Err(format!("Query validation failed; {:?} != {:?}", query_check, result[0..2].to_vec()).into())
        }
    }

    /// Gets the position of a "Turret"
    pub fn turret_pos(
        &mut self,
        turret: ScopeTurret,
    ) -> Result<u8, Box<dyn Error>> {
        let query = format!("HPCr{},1\r", turret as u8);

        let res = self.query_scope(&query)?;
        let response = self.validate(&query, &res.response)?;
        let number = String::from_utf8(response).unwrap();

        Ok(number.parse().unwrap())
    }

    /// Sets the position of a "Turret"
    pub fn set_turret_pos(
        &mut self,
        turret: ScopeTurret,
        position: u8
    ) -> Result<(), Box<dyn Error>> {
        if position > turret.positions() {
            return Err("Position out of range".into());
        }

        let query = format!("HPCR{},{}\r", turret as u8, position);

        self.query_scope(&query)?;

        Ok(())
    }

    /// Gets the light diaphragm aperture
    pub fn ld_pos(&mut self) -> Result<u16, Box<dyn Error>> {
        let query = "HPCs4,1\r";
        let res = self.query_scope(query)?;
        let response = self.validate(&query, &res.response)?;
        let res_string = String::from_utf8(response).unwrap();

        Ok(res_string.parse().unwrap())
    }

    /// Sets the light diaphragm aperture
    pub fn set_ld_pos(&mut self, position: u8) -> Result<(), Box<dyn Error>> {
        let query = format!("HPCS4,{}\r", position);

        self.query_scope(&query)?;

        Ok(())
    }

    /// Gets the focus distance (Z) in steps
    pub fn focus_dist(&mut self) -> Result<i64, Box<dyn Error>> {
        let query = "FPZp\r";

        let res = self.query_scope(query)?;
        let result = self.validate(query, &res.response)?;
        let mut result_24 = hex::decode(&result).unwrap().to_vec();
        result_24.reverse();
        result_24.push(0);
        result_24.reverse();

        let final_number = zeiss_to_i64(i32::from_be_bytes(result_24[0..4].try_into().unwrap()) as i64);
        Ok(final_number)
    }

    /// Sets the focus distance (Z) in steps
    pub fn set_focus_dist(&mut self, distance: i64) -> Result<(), Box<dyn Error>> {
        let output_num = format!("{:06X?}", i64_to_zeiss(distance));

        let mut query = "FPZT".to_string();
        query.push_str(&output_num);
        query.push_str("\r");

        println!("{output_num}\n{query}");

        self.query_scope(&query).unwrap();

        Ok(())
    }

    pub fn focus_limit_upper(&mut self) -> Result<i64, Box<dyn Error>> {
        let query = "FPZu\r";
        let res = self.query_scope(&query)?;
        let response = self.validate(query, &res.response)?;
        let mut result_24 = hex::decode(&response).unwrap().to_vec();
        result_24.reverse();
        result_24.push(0);
        result_24.reverse();
        let value = zeiss_to_i64(i32::from_be_bytes(result_24.try_into().unwrap()) as i64);

        Ok(value)
    }

    pub fn focus_limit_lower(&mut self) -> Result<i64, Box<dyn Error>> {
        let query = "FPZl\r";
        let res = self.query_scope(&query)?;
        let response = self.validate(query, &res.response)?;
        let mut result_24 = hex::decode(&response).unwrap().to_vec();
        result_24.reverse();
        result_24.push(0);
        result_24.reverse();
        let value = zeiss_to_i64(i32::from_be_bytes(result_24[0..4].try_into().unwrap()) as i64);

        Ok(value)
    }

    /// Gets the focus distance (Z) in micrometers (μm)
    pub fn focus_dist_um(&mut self) -> Result<f64, Box<dyn Error>> {
        Ok(Self::STEP_SIZE * self.focus_dist()? as f64)
    }

    /// Sets the focus distance (Z) in micrometers (μm)
    pub fn set_focus_dist_um(&mut self, distance: f64) -> Result<(), Box<dyn Error>> {
        let new_distance = distance / Self::STEP_SIZE;

        self.set_focus_dist(new_distance as i64)
    }
}

pub fn zeiss_to_i64(input_num: i64) -> i64 {
    let mut result = input_num;
    if input_num >= 0x00800000 {
        result = -(0x00FFFFFF - input_num);
        // To convert the numbers into something reasonable, if the 6 bit
        // representation is over 0x800000, then it is made negative and
    }

    result
}

pub fn i64_to_zeiss(input_num: i64) -> i64 {
    let mut result = input_num;
    if input_num < 0 {
        result = 0x00FFFFFF - -(input_num | 0x00F00000);
        // To convert the numbers into something reasonable, if the 6 bit
        // representation is over 0x800000, then it is made negative and
    }

    result
}
