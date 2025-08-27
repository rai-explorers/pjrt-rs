cd third_party
bazel build //:pjrt_headers //:pjrt_protos

rm -rf ../pjrt-sys/include/*
rm -rf ../pjrt-sys/protos/*

tar -xvf bazel-bin/pjrt_headers.tar -C ../pjrt-sys/include
tar -xvf bazel-bin/pjrt_protos.tar -C ../pjrt-sys/protos
