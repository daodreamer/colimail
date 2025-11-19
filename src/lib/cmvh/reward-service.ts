import {
    createWalletClient,
    createPublicClient,
    custom,
    type Address,
    type Hash,
    parseEther,
    keccak256,
    encodeAbiParameters
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
import { walletConnectStore } from "$lib/stores/walletconnect.svelte";

export class RewardService {
    private config: CMVHConfig;

    constructor(config: CMVHConfig) {
        this.config = config;
    }

    private getClient() {
        if (!walletConnectStore.isConnected || !walletConnectStore.address) {
            throw new Error("Wallet not connected via WalletConnect");
        }

        const chain = this.config.network === "arbitrum" ? arbitrum : arbitrumSepolia;

        return walletConnectStore.getWalletClient();
    }

    private getPublicClient() {
        if (!walletConnectStore.isConnected) {
            throw new Error("Wallet not connected via WalletConnect");
        }

        const chain = this.config.network === "arbitrum" ? arbitrum : arbitrumSepolia;

        return createPublicClient({
            chain,
            transport: custom(walletConnectStore["provider"]!)
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
        const address = walletConnectStore.address!;
        const rewardPoolAddress = this.getRewardPoolAddress();

        // Compute email hash: keccak256(abi.encode(subject, from, to))
        const emailHash = keccak256(
            encodeAbiParameters(
                [{ type: "string" }, { type: "string" }, { type: "string" }],
                [emailContent.subject, emailContent.from, emailContent.to]
            )
        );

        // Get public client for reading contract state
        const publicClient = this.getPublicClient();

        // Read wACT token address from RewardPool contract
        const wactAddress = await publicClient.readContract({
            address: rewardPoolAddress,
            abi: [{ name: "wactToken", type: "function", inputs: [], outputs: [{ type: "address" }] }],
            functionName: "wactToken"
        }) as Address;

        // Approve wACT tokens for RewardPool
        const amountWei = parseEther(amount);

        // Check current allowance
        const allowance = await publicClient.readContract({
            address: wactAddress,
            abi: [{ name: "allowance", type: "function", inputs: [{ type: "address" }, { type: "address" }], outputs: [{ type: "uint256" }] }],
            functionName: "allowance",
            args: [address, rewardPoolAddress]
        }) as bigint;

        if (allowance < amountWei) {
            // Need to approve
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
        const address = walletConnectStore.address!;
        const rewardPoolAddress = this.getRewardPoolAddress();

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
        const address = walletConnectStore.address!;
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
