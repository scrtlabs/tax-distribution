const { expect, use, assert } = require("chai");
const {
    Contract,
    getAccountByName,
    polarChai,
    createAccounts,
} = require("secret-polar");
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

describe("Tax Distribution integration tests", () => {
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

    it("Instantiate", async () => {
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

    it("Chagne admin", async () => {
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

        await contract.tx.emergency_withdraw({
            account: contract_owner,
        });
        await expect(
            contract.tx.emergency_withdraw({
                account: beneficiary_1,
            })
        ).to.be.revertedWith("not allowed");

        await expect(
            contract.tx.change_admin(
                { account: beneficiary_1 },
                { new_admin: beneficiary_2.account.address }
            )
        ).to.be.revertedWith("not allowed");

        await contract.tx.change_admin(
            { account: contract_owner },
            { new_admin: beneficiary_1.account.address }
        );

        await expect(
            contract.tx.emergency_withdraw({
                account: contract_owner,
            })
        ).to.be.revertedWith("not allowed");
        await contract.tx.emergency_withdraw({
            account: beneficiary_1,
        });
    });

    it("Set beneficiaries", async () => {
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

        await expect(
            contract.tx.set_beneficiaries(
                { account: contract_owner },
                {
                    beneficiaries: [
                        {
                            address: beneficiary_1.account.address,
                            weight: 600,
                        },
                        {
                            address: beneficiary_2.account.address,
                            weight: 400,
                        },
                        {
                            address: beneficiary_3.account.address,
                            weight: 200,
                        },
                    ],
                    decimal_places_in_weights: 3,
                }
            )
        ).to.be.revertedWith("must be exactly 100");

        let tx_resp = await contract.tx.set_beneficiaries(
            { account: contract_owner },
            {
                beneficiaries: [
                    {
                        address: beneficiary_1.account.address,
                        weight: 600,
                    },
                    {
                        address: beneficiary_2.account.address,
                        weight: 400,
                    },
                ],
                decimal_places_in_weights: 3,
            }
        );

        const event = tx_resp.logs[0].events.find(
            (e) => e.type === "coin_received"
        );
        for (let i = 0; i < event.attributes.length; i += 2) {
            let receiver = event.attributes[i].value;
            let amount = event.attributes[i + 1].value;

            if (receiver === beneficiary_1.account.address) {
                assert(
                    amount === "30000000uscrt",
                    `Unexpected amount for ${receiver}! Expected: 30000000uscrt, Actual: ${amount}`
                );
            } else if (receiver === beneficiary_2.account.address) {
                assert(
                    amount === "50000000uscrt",
                    `Unexpected amount for ${receiver}! Expected: 30000000uscrt, Actual: ${amount}`
                );
            } else if (receiver === beneficiary_3.account.address) {
                assert(
                    amount === "20000000uscrt",
                    `Unexpected amount! Expected: receiver ${receiver} amount: 20000000uscrt, Actual: amount ${amount}`
                );
            }
        }

        await handleTx(
            await cli.tx.bank.send({
                amount: [{ amount: "100000000", denom: "uscrt" }],
                fromAddress: contract_owner.account.address,
                toAddress: contract_info.contractAddress,
            }),
            "send"
        );

        await handleWithdraw(contract, beneficiary_1, "60000000");
        await handleWithdraw(contract, beneficiary_2, "40000000");
        await expect(
            contract.tx.withdraw({
                account: beneficiary_3,
            })
        ).to.be.revertedWith("cannot load beneficiary");
    });

    it("Emergency withdraw", async () => {
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
                amount: [{ amount: "120000000", denom: "uscrt" }],
                fromAddress: contract_owner.account.address,
                toAddress: contract_info.contractAddress,
            }),
            "send"
        );

        await expect(
            contract.tx.emergency_withdraw({ account: beneficiary_1 })
        ).to.be.revertedWith("not allowed");
        const tx_resp = await contract.tx.emergency_withdraw({
            account: contract_owner,
        });

        const event = tx_resp.logs[0].events.find(
            (e) => e.type === "coin_received"
        );
        assert(
            event.attributes.find((a) => a.key === "receiver").value ===
                contract_owner.account.address,
            `Unexpected account! Expected: ${
                contract_owner.account.address
            } Actual: ${
                event.attributes.find((a) => a.key === "receiver").value
            }`
        );
        assert(
            event.attributes.find((a) => a.key === "amount").value ===
                "120000000uscrt",
            `Unexpected amount! Expected: 120000000uscrt Actual: ${
                event.attributes.find((a) => a.key === "amount").value
            }`
        );
    });

    it("Query get beneficiaries", async () => {
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

    it("Query get beneficiary balance", async () => {
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

        await expect(
            contract.query.get_beneficiary_balance({
                address: beneficiary_1.account.address,
            })
        ).to.respondWith({ get_beneficiary_balance: { balance: "30000000" } });
        await expect(
            contract.query.get_beneficiary_balance({
                address: beneficiary_2.account.address,
            })
        ).to.respondWith({ get_beneficiary_balance: { balance: "50000000" } });
        await expect(
            contract.query.get_beneficiary_balance({
                address: beneficiary_3.account.address,
            })
        ).to.respondWith({ get_beneficiary_balance: { balance: "20000000" } });

        await handleWithdraw(contract, beneficiary_1, "1500000", "1500000");

        await expect(
            contract.query.get_beneficiary_balance({
                address: beneficiary_1.account.address,
            })
        ).to.respondWith({ get_beneficiary_balance: { balance: "28500000" } });
        await expect(
            contract.query.get_beneficiary_balance({
                address: beneficiary_2.account.address,
            })
        ).to.respondWith({ get_beneficiary_balance: { balance: "50000000" } });
        await expect(
            contract.query.get_beneficiary_balance({
                address: beneficiary_3.account.address,
            })
        ).to.respondWith({ get_beneficiary_balance: { balance: "20000000" } });

        await handleWithdraw(contract, beneficiary_2, "50000000");

        await expect(
            contract.query.get_beneficiary_balance({
                address: beneficiary_1.account.address,
            })
        ).to.respondWith({ get_beneficiary_balance: { balance: "28500000" } });
        await expect(
            contract.query.get_beneficiary_balance({
                address: beneficiary_2.account.address,
            })
        ).to.respondWith({ get_beneficiary_balance: { balance: "0" } });
        await expect(
            contract.query.get_beneficiary_balance({
                address: beneficiary_3.account.address,
            })
        ).to.respondWith({ get_beneficiary_balance: { balance: "20000000" } });

        await handleWithdraw(contract, beneficiary_1, "28500000");
        await handleWithdraw(contract, beneficiary_3, "20000000");

        await expect(
            contract.query.get_beneficiary_balance({
                address: beneficiary_1.account.address,
            })
        ).to.respondWith({ get_beneficiary_balance: { balance: "0" } });
        await expect(
            contract.query.get_beneficiary_balance({
                address: beneficiary_2.account.address,
            })
        ).to.respondWith({ get_beneficiary_balance: { balance: "0" } });
        await expect(
            contract.query.get_beneficiary_balance({
                address: beneficiary_3.account.address,
            })
        ).to.respondWith({ get_beneficiary_balance: { balance: "0" } });
    });

    it("Query get admin", async () => {
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

        await expect(contract.query.get_admin()).to.respondWith({
            get_admin: { address: contract_owner.account.address },
        });
    });

    it("Sanity run", async () => {
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
