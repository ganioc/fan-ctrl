#!/bin/bash

echo "fan ctrl stop"

I2C_DEV=1
I2C_ADDR=0x4C
REG_ADC=0x4C

SLEEP_SECONDS=10

i2cset -y ${I2C_DEV} ${I2C_ADDR} ${REG_ADC} 0 



