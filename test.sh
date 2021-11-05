cargo build --manifest-path=src/predict/Cargo.toml
cmake -DCMAKE_INSTALL_PREFIX=`pwd` -DCMAKE_BUILD_TYPE=release -DLIBEXECDIR=`pwd` . -DCMAKE_INSTALL_FULL_DATADIR=/usr/share/ -D RELEASE=0
make
sudo make install
