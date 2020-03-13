use ya_model::market::Agreement;

#[test]
fn test_act() {
    let agreement_str = r#"{
  "agreementId": "54033313-db4f-46da-ac9e-6bbf9b635c13",
  "demand": {
    "demandId": "c3c34b8a-7549-44ab-bfa4-b2ef16536aeb",
    "requestorId": "0x1522e27169b611ca9382bcd6b32aae103c4ec14e",
    "properties": {},
    "constraints": "(&\n  (golem.runtime.wasm.wasi.version@v=0.0.0)\n  (golem.inf.mem.gib>0.1)\n  (golem.srv.comp.wasm.task_package=https://github.com/golemfactory/mandelbrot/releases/download/0.2.0/mandelbrot-0.2.0.yimg)\n)\n"
  },
  "offer": {
    "offerId": "54033313-db4f-46da-ac9e-6bbf9b635c13",
    "providerId": "0x1522e27169b611ca9382bcd6b32aae103c4ec14e",
    "properties": {},
    "constraints": "()\n"
  },
  "validTo": "2021-02-25T10:06:08.8272057Z",
  "approvedDate": null,
  "state": "Approved",
  "proposedSignature": null,
  "approvedSignature": null,
  "committedSignature": null
}
"#;

    let agreement: Agreement = serde_json::from_str(agreement_str).unwrap();
}
