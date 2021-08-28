cmake -DCMAKE_INSTALL_PREFIX=`pwd` -DCMAKE_BUILD_TYPE=release -DLIBEXECDIR=`pwd` . -DCMAKE_INSTALL_FULL_DATADIR=/usr/share/
make
sudo make install
