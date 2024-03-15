#!/usr/bin/bash

cargo build --release 
cp target/release/fuel_mileage_api ~/bin/.

cp -r static/ ~/fuelup/.

