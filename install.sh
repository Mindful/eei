cargo build --manifest-path=src/predict/Cargo.toml --release
cmake -DCMAKE_INSTALL_PREFIX=`pwd` -DCMAKE_BUILD_TYPE=release -DLIBEXECDIR=/usr/libexec/ . -DCMAKE_INSTALL_FULL_DATADIR=/usr/share/ -D RELEASE=1
make
sudo make install
