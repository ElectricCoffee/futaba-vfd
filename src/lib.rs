mod futaba_vfd {
    extern crate sysfs_gpio;
    use std::thread::sleep;
    use std::time::Duration;

    pub const CLEAR_DISPLAY: u8 = 0x01;
    pub const SET_CURSOR: u8 = 0x02;
    pub const SET_INTENSITY: u8 = 0x04;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct FutabaVFD {
        clock_pin: sysfs_gpio::Pin,
        data_pin: sysfs_gpio::Pin,
        strobe_pin: sysfs_gpio::Pin,
        column_count: u8,
        intensity: u8,
    }

    impl FutabaVFD {
        pub fn new(clock: u64, data: u64, strobe: u64, column_count: u8, intensity: u8) -> FutabaVFD {
            let clock_pin = sysfs_gpio::Pin::new(clock);
            let data_pin = sysfs_gpio::Pin::new(data);
            let strobe_pin = sysfs_gpio::Pin::new(strobe); 
            FutabaVFD { clock_pin, data_pin, strobe_pin, column_count, intensity }
        }

        // sets up all the pins and marks them as outputs
        pub fn begin(&self) -> sysfs_gpio::Result<()> {
            self.clock_pin.export()?;
            self.data_pin.export()?;
            self.strobe_pin.export()?;

            self.clock_pin.set_direction(sysfs_gpio::Direction::Out)?;
            self.data_pin.set_direction(sysfs_gpio::Direction::Out)?;
            self.strobe_pin.set_direction(sysfs_gpio::Direction::Out)?;

            delay(500);
            self.strobe()?;
            self.send(SET_INTENSITY)?;
            self.send(self.intensity)?;
            self.clear()?;
            Ok(())
        }

        // unexports all the pins
        pub fn end(&self) -> sysfs_gpio::Result<()> {
            self.clock_pin.unexport()?;
            self.data_pin.unexport()?;
            self.strobe_pin.unexport()?;
            Ok(())
        }

        // Lets the user run the display within a closure, rather than having to
        // remember to begin and end the display.
        pub fn with_beginning<F: FnOnce() -> sysfs_gpio::Result<()>>(&self, closure: F) -> sysfs_gpio::Result<()> {
            self.begin()?;
            match closure() {
                Ok(()) => {
                    try!(self.end());
                    Ok(())
                },
                Err(e) => {
                    self.end()?;
                    Err(e)
                }
            }
        }

        pub fn clear(&self) -> sysfs_gpio::Result<()> {
            self.send(CLEAR_DISPLAY)
        }

        pub fn home(&self) -> sysfs_gpio::Result<()> {
            self.set_cursor(0, 0)
        }

        pub fn set_cursor(&self, col: u8, row: u8) -> sysfs_gpio::Result<()> {
            self.send(SET_CURSOR)?;
            self.send(row * self.column_count + col + 1)
        }

        pub fn send(&self, data: u8) -> sysfs_gpio::Result<()> {
            let mut mask: u8 = 0x80;
            for _ in 0 .. 8 {
                self.clock_pin.set_value(1)?;
                delay_micros(15);
                self.data_pin.set_value(!!(data & mask))?;
                delay_micros(5);
                self.clock_pin.set_value(0)?;

                mask = mask >> 1;
            }
            Ok(())
        }

        pub fn strobe(&self) -> sysfs_gpio::Result<()> {
            self.clock_pin.set_value(1)?;
            self.data_pin.set_value(1)?;

            self.strobe_pin.set_value(1)?;
            delay(5);
            self.strobe_pin.set_value(0)?;
            delay(5);
            Ok(())
        }
    }

    // convenience function
    fn delay(time: u64) {
        sleep(Duration::from_millis(time));
    }

    fn delay_micros(time: u32) {
        sleep(Duration::new(0, time * 1000));
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
