const { expect, use, assert } = require("chai");
const { Contract, getAccountByName, polarChai } = require("secret-polar");
const { Wallet, SecretNetworkClient, TxResultCode } = require("secretjs");

use(polarChai);

const createCli = async () => {
    let url = new URL("http://localhost/");
    // let url = new URL(rest_endpoint);
    url.port = "9091";
    let rest_endpoint = url.toString();

    const wallet = new Wallet(
        "grant rice replace explain federal release fix clever romance raise often wild taxi quarter soccer fiber love must tape steak together observe swap guitar"
    );
    const accAddress = wallet.address;

    return await SecretNetworkClient.create({
        grpcWebUrl: rest_endpoint,
        chainId: "secretdev-1",
        wallet: wallet,
        walletAddress: accAddress,
    });
};

function handleTx(tx, txName) {
    txName = txName || "transaction";

    console.log(`Gas used by ${txName}: ${tx.gasUsed}`);
    if (tx.code !== TxResultCode.Success) {
        console.log(JSON.stringify(tx.jsonLog || tx));
        throw new ComputeError(tx, `Failed to run ${txName}`);
    }
    return tx;
}

async function handleWithdraw(
    contract,
    account,
    expectedAmount,
    specifyAmount
) {
    expectedAmount = expectedAmount + "uscrt";
    let tx_resp = await contract.tx.withdraw(
        {
            account: account,
            transferAmount: [],
        },
        { amount: specifyAmount }
    );

    const event = tx_resp.logs[0].events.find(
        (e) => e.type === "coin_received"
    );
    assert(
        event.attributes.find((a) => a.key === "receiver").value ===
            account.account.address,
        `Unexpected account! Expected: ${account.account.address} Actual: ${
            event.attributes.find((a) => a.key === "receiver").value
        }`
    );
    assert(
        event.attributes.find((a) => a.key === "amount").value ===
            expectedAmount,
        `Unexpected amount! Expected: ${expectedAmount} Actual: ${
            event.attributes.find((a) => a.key === "amount").value
        }`
    );
}

describe("sample_project", () => {
    async function setup() {
        const contract_owner = getAccountByName("account_0");
        const beneficiary_1 = getAccountByName("account_1");
        const beneficiary_2 = getAccountByName("account_2");
        const beneficiary_3 = getAccountByName("account_3");
        const contract = new Contract("tax-distribution");
        await contract.parseSchema();

        await contract.deploy(contract_owner, {
            amount: [{ denom: "uscrt", amount: "25000" }],
            gas: "2000000",
        });
        const cli = await createCli();

        return {
            cli,
            contract_owner,
            contract,
            beneficiary_1,
            beneficiary_2,
            beneficiary_3,
        };
    }

    it("deploy and init", async () => {
        this._timeout = 1000000000;
        const {
            cli,
            contract_owner,
            contract,
            beneficiary_1,
            beneficiary_2,
            beneficiary_3,
        } = await setup();

        const contract_info = await contract.instantiate(
            {
                beneficiaries: [
                    {
                        address: beneficiary_1.account.address,
                        weight: 300,
                    },
                    {
                        address: beneficiary_2.account.address,
                        weight: 500,
                    },
                    {
                        address: beneficiary_3.account.address,
                        weight: 200,
                    },
                ],
                decimal_places_in_weights: 3,
            },
            "deploy test",
            contract_owner
        );
        console.log(
            `Successfully deployed contract: ${contract_info.contractAddress}`
        );

        await expect(contract.query.get_beneficiaries()).to.respondWith({
            get_beneficiaries: {
                beneficiaries: [
                    {
                        address: beneficiary_1.account.address,
                        weight: 300,
                        withdrawn: "0",
                    },
                    {
                        address: beneficiary_2.account.address,
                        weight: 500,
                        withdrawn: "0",
                    },
                    {
                        address: beneficiary_3.account.address,
                        weight: 200,
                        withdrawn: "0",
                    },
                ],
            },
        });
    });

    it("sanity", async () => {
        const {
            cli,
            contract_owner,
            contract,
            beneficiary_1,
            beneficiary_2,
            beneficiary_3,
        } = await setup();

        const contract_info = await contract.instantiate(
            {
                beneficiaries: [
                    {
                        address: beneficiary_1.account.address,
                        weight: 300,
                    },
                    {
                        address: beneficiary_2.account.address,
                        weight: 500,
                    },
                    {
                        address: beneficiary_3.account.address,
                        weight: 200,
                    },
                ],
                decimal_places_in_weights: 3,
            },
            "deploy test",
            contract_owner
        );
        console.log(
            `Successfully deployed contract: ${contract_info.contractAddress}`
        );

        await handleTx(
            await cli.tx.bank.send({
                amount: [{ amount: "100000000", denom: "uscrt" }],
                fromAddress: contract_owner.account.address,
                toAddress: contract_info.contractAddress,
            }),
            "send"
        );

        await handleWithdraw(contract, beneficiary_1, "30000000");
        await handleWithdraw(contract, beneficiary_2, "10000000", "10000000");

        await handleTx(
            await cli.tx.bank.send({
                amount: [{ amount: "1000000000000", denom: "uscrt" }],
                fromAddress: contract_owner.account.address,
                toAddress: contract_info.contractAddress,
            }),
            "send"
        );

        await handleWithdraw(contract, beneficiary_1, "300000000000");
        await handleWithdraw(contract, beneficiary_2, "500040000000");
        await handleWithdraw(contract, beneficiary_3, "200020000000");

        await handleTx(
            await cli.tx.bank.send({
                amount: [{ amount: "123456123456", denom: "uscrt" }],
                fromAddress: contract_owner.account.address,
                toAddress: contract_info.contractAddress,
            }),
            "send"
        );

        await handleWithdraw(contract, beneficiary_1, "37036837036");
        await handleWithdraw(contract, beneficiary_2, "61728061728");
        await handleWithdraw(contract, beneficiary_3, "24691224691");
    });
});
