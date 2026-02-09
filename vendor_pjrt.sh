cd third_party
bazelisk build //:pjrt_headers //:pjrt_protos

rm -rf ../pjrt-sys/include/*
rm -rf ../pjrt-sys/protos/*

tar -xf bazel-bin/pjrt_include.tar -C ../pjrt-sys/include
tar -xf bazel-bin/pjrt_protos.tar -C ../pjrt-sys/protos
cd ..