set -ev

cmake -Bbuild --install-prefix /usr -DCMAKE_BUILD_TYPE=Release
sudo cmake --build build -t install
