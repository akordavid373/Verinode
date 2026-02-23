import { CrossChainService } from '../../services/crossChain/crossChainService';
import { BridgeService } from '../../services/crossChain/bridgeService';
import { GasOptimizer } from '../../services/crossChain/gasOptimizer';

const crossChainService = new CrossChainService();
const bridgeService = new BridgeService();
const gasOptimizer = new GasOptimizer();

export const crossChainResolvers = {
  Query: {
    supportedChains: () => crossChainService.getSupportedChains(),
    
    walletInfo: (_: any, { address, chainId }: { address: string; chainId: number }) =>
      crossChainService.getWalletInfo(address, chainId),
    
    crossChainTransfer: (_: any, { transferId }: { transferId: string }) =>
      crossChainService.getTransferStatus(transferId),
    
    crossChainTransfers: (_: any, { status }: { status?: string }) =>
      crossChainService.getAllPendingTransfers(),
    
    optimizeGas: (_: any, { fromChain, toChain, amount }: { fromChain: number; toChain: number; amount: string }) =>
      gasOptimizer.optimizeGas(fromChain, toChain, amount),
  },

  Mutation: {
    initiateCrossChainTransfer: async (_: any, args: any) => {
      return await crossChainService.initiateTransfer(args);
    },
    
    completeCrossChainTransfer: async (_: any, { transferId }: { transferId: string }) => {
      return await crossChainService.completeTransfer(transferId);
    },
    
    switchChain: async (_: any, { targetChainId }: { targetChainId: number }, context: any) => {
      if (!context.user) {
        throw new Error('Authentication required');
      }
      return await crossChainService.switchChain(context.user.address, targetChainId);
    },
  },

  Subscription: {
    crossChainTransferUpdated: {
      subscribe: (_: any, { transferId }: { transferId?: string }) => {
        // Implementation for real-time transfer updates
        return {
          [Symbol.asyncIterator]: () => ({
            next: async () => ({ value: null, done: true })
          })
        };
      }
    }
  }
};
