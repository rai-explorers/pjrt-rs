module {
    func.func @main(%arg0: tensor<f32>) -> tensor<f32> {
        %0 = "mhlo.copy"(%arg0) : (tensor<f32>) -> tensor<f32>
        %1 = mhlo.constant dense<1.000000e+00> : tensor<f32>
        %2 = mhlo.add %0, %1 : tensor<f32>
        return %2 : tensor<f32>
    }
}