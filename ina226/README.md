# a driver for the INA226 voltage an current sensor
based on [INA226_WE](https://github.com/wollewald/INA226_WE)
made for esp32c3, not sure if it works on others or how to make it work

minimal documentation so far, but its basically the same as above
sorry the minimal documentation is so bad i mostly copied INA226_WE and only learned about the stuff necessary for my application (so i dont understand most of the config)
the datasheet should explain a lot of it

please contribute if you think you could improve the library, ESPECIALLy the documentation

### usage
create a new instance of the `Ina226` struct and run init on it
this can be dropped, but if the ina226 has already been initialized use `Ina226::new_initialized()` instead to sync the config

always run `Ina226::read_and_clear_flags()` before reading measurments