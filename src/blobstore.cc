#include "ruff-hnt-rs/include/blobstore.h"
#include "ruff-hnt-rs/src/main.rs.h"

#include <algorithm>
#include <cstdint>
#include <cstdio>
#include <functional>
#include <set>
#include <string>
#include <unordered_map>

#include <iostream>

#include "ruff-hnt-rs/include/ADS1X15_TLA2024.h"
namespace org {
namespace blobstore {

// Toy implementation of an in-memory blobstore.
//
// In reality the implementation of BlobstoreClient could be a large complex C++
// library.
class BlobstoreClient::impl {
  friend BlobstoreClient;

  TLA2024 adc;
};

BlobstoreClient::BlobstoreClient() : impl(new class BlobstoreClient::impl) {}

// Upload a new blob and return a blobid that serves as a handle to the blob.
uint16_t BlobstoreClient::read(std::uint8_t channel) const {
  std::cout<< "in read" << channel;
  return impl->adc.readADC_SingleEnded(channel);
}


std::unique_ptr<BlobstoreClient> new_blobstore_client() {
  return std::make_unique<BlobstoreClient>();
}

} // namespace blobstore
} // namespace org
