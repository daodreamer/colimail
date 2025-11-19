import {
    createWalletClient,
    custom,
    type Address,
    type Hash,
    parseEther
} from "viem";
import { arbitrum, arbitrumSepolia } from "viem/chains";
import {
    REWARD_POOL_ABIS,
    getRewardInfo,
    getUserRewards,
    findRewardForEmail,
    getUserStats
} from "./blockchain";
import { NETWORK_CONFIG, type CMVHConfig, type EmailContent } from "./types";
import { walletStore } from "$lib/stores/wallet.svelte";

export class RewardService {
    private config: CMVHConfig;

    constructor(config: CMVHConfig) {
        this.config = config;
    }

    private getClient() {
        if (typeof window === "undefined" || !window.ethereum) {
            throw new Error("No wallet found");
        }

        const chain = this.config.network === "arbitrum" ? arbitrum : arbitrumSepolia;

        return createWalletClient({
            chain,
            transport: custom(window.ethereum)
        });
    }

    private getRewardPoolAddress(): Address {
        const networkConfig = NETWORK_CONFIG[this.config.network];
        const address = this.config.rewardPoolAddress || networkConfig.rewardPoolAddress;

        if (!address) throw new Error("Reward pool address not configured");
        return address as Address;
    }

    /**
     * Create a new reward
     */
    async createReward(
        recipient: string,
        amount: string, // in wACT (ether units)
        emailContent: EmailContent,
        expiryDuration: number = 30 * 24 * 60 * 60 // 30 days default
    ): Promise<Hash> {
        const client = this.getClient();
        const [address] = await client.requestAddresses();
        const rewardPoolAddress = this.getRewardPoolAddress();

        // First we need to compute the email hash
        // We can use the public client from blockchain.ts or just assume the contract will verify it
        // But createReward takes emailHash as input.
        // We need to compute it exactly as the contract does.
        // For now, let's rely on the fact that we can compute it locally if we had the logic,
        // but since we don't want to duplicate logic, let's fetch it from the Verifier contract via blockchain.ts helper?
        // Actually, `findRewardForEmail` in blockchain.ts computes it.
        // Let's duplicate the read-only call to get the hash first.

        // TODO: Optimize this by computing locally using viem's keccak256/encodeAbiParameters
        // For now, we'll just pass a placeholder or fetch it if we can.
        // Wait, `createReward` in contract takes `emailHash`.
        // We need to get this hash.
        // Let's add a helper in blockchain.ts to just get the hash?
        // Or just use the public client here to read it from Verifier.

        // Let's assume we have a helper or we can just calculate it.
        // Since I didn't expose a public helper for hash, I'll use the public client pattern again here or just implement local hashing.
        // Local hashing is better for performance.
        // keccak256(abi.encode(subject, from, to))

        // But wait, I need to be sure about the encoding.
        // The contract says: `keccak256(abi.encode(subject, from, to))`
        // In viem: keccak256(encodeAbiParameters([{type: 'string'}, {type: 'string'}, {type: 'string'}], [subject, from, to]))

        const { keccak256, encodeAbiParameters } = await import("viem");

        const emailHash = keccak256(
            encodeAbiParameters(
                [{ type: "string" }, { type: "string" }, { type: "string" }],
                [emailContent.subject, emailContent.from, emailContent.to]
            )
        );

        // Check allowance? The contract does `wactToken.safeTransferFrom`.
        // We need to approve wACT first!
        // This is a missing step in the plan.
        // I need the wACT token address.
        // It's not in the config.
        // I should probably ask the user or find it in the contract code/events.
        // For now, I will assume the user has approved or I'll add an approve step if I can find the token address.
        // The contract has `wactToken` public variable. I can read it.

        // Let's read wACT address from contract first.
        const publicClient = await import("viem").then(m => m.createPublicClient({
            chain: this.config.network === "arbitrum" ? arbitrum : arbitrumSepolia,
            transport: custom(window.ethereum!)
        }));

        const wactAddress = await publicClient.readContract({
            address: rewardPoolAddress,
            abi: [{ name: "wactToken", type: "function", inputs: [], outputs: [{ type: "address" }] }],
            functionName: "wactToken"
        }) as Address;

        // Approve wACT
        const amountWei = parseEther(amount);

        // Check allowance
        const allowance = await publicClient.readContract({
            address: wactAddress,
            abi: [{ name: "allowance", type: "function", inputs: [{ type: "address" }, { type: "address" }], outputs: [{ type: "uint256" }] }],
            functionName: "allowance",
            args: [address, rewardPoolAddress]
        }) as bigint;

        if (allowance < amountWei) {
            const approveTx = await client.writeContract({
                address: wactAddress,
                abi: [{ name: "approve", type: "function", inputs: [{ type: "address" }, { type: "uint256" }], outputs: [{ type: "bool" }] }],
                functionName: "approve",
                args: [rewardPoolAddress, amountWei],
                account: address
            });
            await publicClient.waitForTransactionReceipt({ hash: approveTx });
        }

        // Create Reward
        const hash = await client.writeContract({
            address: rewardPoolAddress,
            abi: REWARD_POOL_ABIS.createReward,
            functionName: "createReward",
            args: [
                recipient as Address,
                amountWei,
                emailHash,
                emailContent.subject,
                emailContent.from,
                emailContent.to,
                BigInt(expiryDuration)
            ],
            account: address
        });

        return hash;
    }

    /**
     * Claim a reward
     */
    async claimReward(
        rewardId: string,
        emailContent: EmailContent,
        signature: string
    ): Promise<Hash> {
        const client = this.getClient();
        const [address] = await client.requestAddresses();
        const rewardPoolAddress = this.getRewardPoolAddress();

        const { keccak256, encodeAbiParameters } = await import("viem");
        const emailHash = keccak256(
            encodeAbiParameters(
                [{ type: "string" }, { type: "string" }, { type: "string" }],
                [emailContent.subject, emailContent.from, emailContent.to]
            )
        );

        const hash = await client.writeContract({
            address: rewardPoolAddress,
            abi: REWARD_POOL_ABIS.claimReward,
            functionName: "claimReward",
            args: [
                rewardId as `0x${string}`,
                emailHash,
                signature as `0x${string}`,
                emailContent.subject,
                emailContent.from,
                emailContent.to
            ],
            account: address
        });

        return hash;
    }

    /**
     * Cancel a reward
     */
    async cancelReward(rewardId: string): Promise<Hash> {
        const client = this.getClient();
        const [address] = await client.requestAddresses();
        const rewardPoolAddress = this.getRewardPoolAddress();

        const hash = await client.writeContract({
            address: rewardPoolAddress,
            abi: REWARD_POOL_ABIS.cancelReward,
            functionName: "cancelReward",
            args: [rewardId as `0x${string}`],
            account: address
        });

        return hash;
    }

    // Re-export read functions for convenience
    async getRewardInfo(rewardId: string) {
        return getRewardInfo(rewardId, this.config);
    }

    async getUserRewards(user: string, asRecipient: boolean) {
        return getUserRewards(user, asRecipient, this.config);
    }

    async getUserStats(user: string) {
        return getUserStats(user, this.config);
    }

    async findRewardForEmail(recipient: string, content: EmailContent) {
        return findRewardForEmail(recipient, content, this.config);
    }
}
