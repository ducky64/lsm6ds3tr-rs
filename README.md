# lsm6ds3tr-rs
A no-std, no-alloc, embedded-hal-async driver for the LSM6DS3TR-C IMU with raw and normalized output

I2C only. Architecturally extensible to SPI, but I don't have SPI hardware to test against.

Tested on nRF52840, but should be compatible with anything implementing embedded-hal-async.

## Example

```rust
let mut imu = Lsm6ds3tr::new_i2c(i2c);
if !imu.check().await.unwrap() {
    panic!("IMU ID check failed");
}
imu.reset().await.unwrap();
dev.config_xl(lsm6ds3trc::OdrXl::new_at_least(5), lsm6ds3trc::FsXl::G4)
    .await.unwrap();
dev.config_g(lsm6ds3trc::OdrG::new_at_least(100), lsm6ds3trc::FsG::Dps500)
    .await.unwrap();
let (x, y, z) = imu.read_xl_g().await.unwrap();
info!("IMU: x={}, y={}, z={}", x, y, z);
let (pitch, roll, yaw) = imu.read_g_dps().await.unwrap();
info!("IMU: p={}, r={}, y={}", pitch, roll, yaw);
```
