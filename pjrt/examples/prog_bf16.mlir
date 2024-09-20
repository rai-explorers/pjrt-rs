module {
    func.func @main(%arg0: tensor<bf16>) -> tensor<bf16> {
        %0 = "mhlo.copy"(%arg0) : (tensor<bf16>) -> tensor<bf16>
        %1 = mhlo.constant dense<1.000000e+00> : tensor<bf16>
        %2 = mhlo.add %0, %1 : tensor<bf16>
        return %2 : tensor<bf16>
    }
}