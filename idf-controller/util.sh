# mount container
# docker run --mount type=bind,source="$(pwd)",target=/workspace,consistency=cached -it espressif/rust-std-training:latest /bin/bash


# flash esp
# espflash flash --monitor target/riscv32imc-esp-espidf/release/idf-controller
