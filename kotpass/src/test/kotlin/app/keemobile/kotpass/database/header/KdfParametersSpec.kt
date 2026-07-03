package app.keemobile.kotpass.database.header

import app.keemobile.kotpass.constants.Const
import app.keemobile.kotpass.constants.KdfConst
import io.kotest.core.spec.style.DescribeSpec
import io.kotest.matchers.shouldBe
import io.kotest.matchers.types.shouldBeInstanceOf
import okio.ByteString.Companion.toByteString

class KdfParametersSpec : DescribeSpec({

    describe("KDF parameters") {
        it("Reads legacy KDBX3 AES KDF UUID from KDBX4 variant dictionaries") {
            val seed = byteArrayOf(0x1, 0x2, 0x3).toByteString()
            val data = VariantDictionary.writeToByteString(
                mapOf(
                    KdfConst.Keys.Uuid to VariantItem.Bytes(KdfParameters.Aes.LegacyUuid),
                    KdfConst.Keys.Rounds to VariantItem.UInt64(10UL),
                    KdfConst.Keys.SaltOrSeed to VariantItem.Bytes(seed)
                )
            )

            val parameters = KdfParameters.readFrom(data)

            parameters.shouldBeInstanceOf<KdfParameters.Aes>()
            parameters.rounds shouldBe 10UL
            parameters.seed shouldBe seed
        }

        it("Writes the KDBX4 AES KDF UUID") {
            val data = KdfParameters.Aes(
                rounds = 10UL,
                seed = byteArrayOf(0x1, 0x2, 0x3).toByteString()
            ).writeToByteString()

            val uuid = (VariantDictionary.readFrom(data)[KdfConst.Keys.Uuid] as VariantItem.Bytes).value

            uuid shouldBe Const.bytes(
                0x7C, 0x02, 0xBB, 0x82, 0x79, 0xA7, 0x4A, 0xC0,
                0x92, 0x7D, 0x11, 0x4A, 0x00, 0x64, 0x82, 0x38
            )
        }
    }
})
