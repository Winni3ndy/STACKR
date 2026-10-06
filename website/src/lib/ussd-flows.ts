export interface UssdNode {
  text: string;
  options?: Record<string, UssdNode>;
  end?: boolean;
}

export const ussdTree: UssdNode = {
  text: `Welcome to Stackr

1. Send Money
2. Pay Merchant
3. Buy Airtime/Data
4. Pay Bills
5. Check Balance
6. Swap Tokens
7. Withdraw to Bank
8. Deposit
9. My Wallet`,
  options: {
    "1": {
      text: "Enter recipient phone number:",
      options: {
        "*": {
          text: "Enter amount (USDC):",
          options: {
            "*": {
              text: `Confirm transfer:
To: +250781234567
Amount: 10.00 USDC

Enter PIN to confirm:`,
              options: {
                "*": {
                  text: "Transfer successful!\n\n10.00 USDC sent to +250781234567\nTx: 3d8f...a91c\n\nNew balance: 240.00 USDC",
                  end: true,
                },
              },
            },
          },
        },
      },
    },
    "2": {
      text: "Enter merchant code:",
      options: {
        "*": {
          text: "Enter amount (USDC):",
          options: {
            "*": {
              text: `Payment to: QuickMart
Amount: 5.00 USDC

Enter PIN to confirm:`,
              options: {
                "*": {
                  text: "Payment successful!\n\n5.00 USDC paid to QuickMart\nRef: MR-7829\n\nNew balance: 235.00 USDC",
                  end: true,
                },
              },
            },
          },
        },
      },
    },
    "3": {
      text: `Buy Airtime/Data

1. Airtime
2. Data Bundle`,
      options: {
        "1": {
          text: "Enter phone number for airtime:",
          options: {
            "*": {
              text: "Enter amount (RWF):",
              options: {
                "*": {
                  text: `Buy airtime:
Phone: +250781234567
Amount: 1,000 RWF (~0.73 USDC)

Enter PIN to confirm:`,
                  options: {
                    "*": {
                      text: "Airtime purchased!\n\n1,000 RWF airtime sent to +250781234567\nCharged: 0.73 USDC\n\nNew balance: 234.27 USDC",
                      end: true,
                    },
                  },
                },
              },
            },
          },
        },
        "2": {
          text: "Enter phone number for data bundle:",
          options: {
            "*": {
              text: "Enter amount (RWF):",
              options: {
                "*": {
                  text: "Data bundle purchased!\n\nCharged: 1.46 USDC\n\nNew balance: 232.81 USDC",
                  end: true,
                },
              },
            },
          },
        },
      },
    },
    "4": {
      text: `Pay Bills

1. Electricity
2. Cable TV
3. Internet
4. Betting`,
      options: {
        "1": {
          text: "Enter meter number:",
          options: {
            "*": {
              text: "Enter amount (RWF):",
              options: {
                "*": {
                  text: "Enter PIN to confirm:",
                  options: {
                    "*": {
                      text: "Bill payment successful!\n\nElectricity payment: 5,000 RWF\nCharged: 3.65 USDC\nToken: 4829-1847-2938-4719\n\nNew balance: 229.16 USDC",
                      end: true,
                    },
                  },
                },
              },
            },
          },
        },
        "2": {
          text: "Enter smart card number:",
          options: {
            "*": {
              text: "Enter amount (RWF):",
              options: {
                "*": {
                  text: "Cable TV payment successful!\n\nCharged: 7.30 USDC\n\nNew balance: 221.86 USDC",
                  end: true,
                },
              },
            },
          },
        },
        "3": {
          text: "Enter account number:",
          options: {
            "*": {
              text: "Internet payment successful!\n\nCharged: 14.60 USDC\n\nNew balance: 207.26 USDC",
              end: true,
            },
          },
        },
        "4": {
          text: "Enter betting account ID:",
          options: {
            "*": {
              text: "Betting top-up successful!\n\nCharged: 3.65 USDC\n\nNew balance: 203.61 USDC",
              end: true,
            },
          },
        },
      },
    },
    "5": {
      text: "Your Balance\n\nUSDC: 250.00\nXLM: 5.00\n\nDaily spent: 0.00 / 500.00 USDC\nKYC Tier: 1",
      end: true,
    },
    "6": {
      text: `Swap Tokens

1. USDC → XLM
2. XLM → USDC`,
      options: {
        "1": {
          text: "Enter USDC amount to swap:",
          options: {
            "*": {
              text: `Swap preview:
Send: 10.00 USDC
Receive: ~85.47 XLM (est.)

Enter PIN to confirm:`,
              options: {
                "*": {
                  text: "Swap successful!\n\nSent: 10.00 USDC\nReceived: 85.47 XLM\nTx: 9f2a...c4d1\n\nNew USDC balance: 240.00",
                  end: true,
                },
              },
            },
          },
        },
        "2": {
          text: "Enter XLM amount to swap:",
          options: {
            "*": {
              text: "Swap successful!\n\nSent: 50.00 XLM\nReceived: 5.85 USDC\n\nNew USDC balance: 255.85",
              end: true,
            },
          },
        },
      },
    },
    "7": {
      text: `Withdraw to Bank

Enter USDC amount:`,
      options: {
        "*": {
          text: "Enter PIN to confirm:",
          options: {
            "*": {
              text: "Withdrawal initiated!\n\nAmount: 50.00 USDC\nYou will receive: ~68,500 RWF\nMethod: Mobile Money\nStatus: Processing\n\nYou'll receive an SMS when complete.",
              end: true,
            },
          },
        },
      },
    },
    "8": {
      text: `Deposit

Enter amount in RWF:`,
      options: {
        "*": {
          text: "Deposit initiated!\n\nAmount: 50,000 RWF\nYou will receive: ~36.50 USDC\nMethod: Mobile Money\nStatus: Pending\n\nFollow the payment prompt on your phone.",
          end: true,
        },
      },
    },
    "9": {
      text: `My Wallet

Address: GBXY...K4NP
Network: Stellar Mainnet
Assets: USDC, XLM

1. Copy Address
0. Back`,
      options: {
        "1": {
          text: "Your full wallet address:\n\nGBXYZ2KCJQPG3RV7PMHZCFN4AOXK4NP\n\nShare this to receive USDC.",
          end: true,
        },
        "0": {
          text: `Welcome to Stackr

1. Send Money
2. Pay Merchant
3. Buy Airtime/Data
4. Pay Bills
5. Check Balance
6. Swap Tokens
7. Withdraw to Bank
8. Deposit
9. My Wallet`,
        },
      },
    },
  },
};
