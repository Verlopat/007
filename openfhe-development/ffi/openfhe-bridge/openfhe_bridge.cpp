#include "openfhe.h"

using namespace lbcrypto;

extern "C" {

void* create_threshold_context() {
    auto cc = CryptoContextFactory<DCRTPoly>::genCryptoContextBFVrns(1024);
    cc->Enable(ENCRYPTION) | Enable(LEXICON);        // | Enable(SCHEMES::BFVrnsTHRESHOLD);
    return static_cast<void*>(cc.release());
}

}
