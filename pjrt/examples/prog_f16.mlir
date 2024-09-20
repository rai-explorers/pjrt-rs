module {
    func.func @main(%arg0: tensor<f16>) -> tensor<f16> {
        %0 = "mhlo.copy"(%arg0) : (tensor<f16>) -> tensor<f16>
        %1 = mhlo.constant dense<1.000000e+00> : tensor<f16>
        %2 = mhlo.add %0, %1 : tensor<f16>
        return %2 : tensor<f16>
    }
}