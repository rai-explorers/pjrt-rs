cd third_party
bazel build //:pjrt_headers //:pjrt_protos

mv bazel-bin/include ../pjrt-sys/include
mv bazel-bin/protos ../pjrt-sys/protos