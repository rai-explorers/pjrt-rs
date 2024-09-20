module {
    func.func @main(%arg0: tensor<complex<f32>>) -> tensor<complex<f32>> {
        %0 = "mhlo.copy"(%arg0) : (tensor<complex<f32>>) -> tensor<complex<f32>>
        %1 = mhlo.constant dense<(1.000000e+00, 1.000000e+00)> : tensor<complex<f32>>
        %2 = mhlo.add %0, %1 : tensor<complex<f32>>
        return %2 : tensor<complex<f32>>
    }
}