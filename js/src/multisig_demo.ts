export type MultisigDemo = {
  "version": "0.1.0",
  "name": "multisig_demo",
  "instructions": [
    {
      "name": "newMultisig",
      "docs": [
        "Initialize a new [MultisigWallet]."
      ],
      "accounts": [
        {
          "name": "base",
          "isMut": false,
          "isSigner": true,
          "docs": [
            "Entropy to ensure a unique address.",
            "Improves on a design flaw in Project Serum's multisig implementation."
          ]
        },
        {
          "name": "payer",
          "isMut": true,
          "isSigner": true,
          "docs": [
            "Funds the creation of the wallet. Does not need to be contained in [members]."
          ]
        },
        {
          "name": "multisigWallet",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The PDA which stores relevant state, and which also signs approved transactions",
            "during their execution."
          ]
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "threshold",
          "type": "u16"
        },
        {
          "name": "members",
          "type": {
            "vec": "publicKey"
          }
        }
      ]
    },
    {
      "name": "newTransaction",
      "docs": [
        "Initialize a new [MultisigTransaction]."
      ],
      "accounts": [
        {
          "name": "proposer",
          "isMut": true,
          "isSigner": true,
          "docs": [
            "Must be a member of the given [multisig_wallet]."
          ]
        },
        {
          "name": "multisigWallet",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The wallet whose address is intended to sign the transaction",
            "upon execution. Only members of this wallet can sign for tx approval."
          ]
        },
        {
          "name": "transaction",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "instructions",
          "type": {
            "vec": {
              "defined": "Instruction"
            }
          }
        }
      ]
    },
    {
      "name": "approve",
      "docs": [
        "Approve a [MultisigTransaction] for execution."
      ],
      "accounts": [
        {
          "name": "member",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "multisigWallet",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "transaction",
          "isMut": true,
          "isSigner": false
        }
      ],
      "args": []
    },
    {
      "name": "unapprove",
      "docs": [
        "Cancel approval for a [MultisigTransaction]."
      ],
      "accounts": [
        {
          "name": "member",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "multisigWallet",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "transaction",
          "isMut": true,
          "isSigner": false
        }
      ],
      "args": []
    },
    {
      "name": "execute",
      "docs": [
        "Execute a [MultisigTransaction], iff it has enough approvals and hasn't",
        "yet been executed."
      ],
      "accounts": [
        {
          "name": "member",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "multisigWallet",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "transaction",
          "isMut": true,
          "isSigner": false
        }
      ],
      "args": []
    },
    {
      "name": "changeThreshold",
      "accounts": [
        {
          "name": "multisigWallet",
          "isMut": true,
          "isSigner": true
        }
      ],
      "args": [
        {
          "name": "threshold",
          "type": "u16"
        }
      ]
    },
    {
      "name": "changeMembers",
      "accounts": [
        {
          "name": "multisigWallet",
          "isMut": true,
          "isSigner": true
        }
      ],
      "args": [
        {
          "name": "members",
          "type": {
            "vec": "publicKey"
          }
        }
      ]
    }
  ],
  "accounts": [
    {
      "name": "multisigWallet",
      "docs": [
        "PDA that represents an M of N multisig signer, and all relevant metadata."
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "base",
            "docs": [
              "Base used to derive."
            ],
            "type": "publicKey"
          },
          {
            "name": "members",
            "docs": [
              "Members able to create and approve a [MultisigTransaction] owned by this",
              "multisig."
            ],
            "type": {
              "vec": "publicKey"
            }
          },
          {
            "name": "threshold",
            "docs": [
              "At least this many members needed to approve [MultisigTransaction]",
              "before it can be executed."
            ],
            "type": "u16"
          },
          {
            "name": "txNonce",
            "docs": [
              "To ensure uniqueness of [MultisigTransaction] PDA address."
            ],
            "type": "u64"
          },
          {
            "name": "memberSetSeqno",
            "docs": [
              "This is used to make sure that a transaction created with owner set X",
              "cannot be approved by a new owner set Y. This handles the edge cases",
              "where a user approves a transaction and then leaves multisig membership."
            ],
            "type": "u32"
          },
          {
            "name": "bump",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "multisigTransaction",
      "docs": [
        "PDA that represents a proposed transaction, and contains all relevant metadata."
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "instructions",
            "docs": [
              "The instruction set of the transaction to be executed."
            ],
            "type": {
              "vec": {
                "defined": "Instruction"
              }
            }
          },
          {
            "name": "multisigWallet",
            "docs": [
              "The [MultisigWallet] account this transaction belongs to."
            ],
            "type": "publicKey"
          },
          {
            "name": "approved",
            "docs": [
              "Keeps track of which accounts approved the transaction, and when",
              "their last approval occurred.",
              "`approved[i]` is Some iff `[MultisigWallet.members[i]]` signed the transaction."
            ],
            "type": {
              "vec": {
                "option": "i64"
              }
            }
          },
          {
            "name": "memberSetSeqno",
            "docs": [
              "Saved to ensure that no approvals occur on a different set of memberships."
            ],
            "type": "u32"
          },
          {
            "name": "createdAt",
            "docs": [
              "Unix timestamp at time of the [MultisigTransaction] account's creation."
            ],
            "type": "i64"
          },
          {
            "name": "proposer",
            "docs": [
              "The account that executed the [Transaction]."
            ],
            "type": "publicKey"
          },
          {
            "name": "executor",
            "docs": [
              "The account that executed the [Transaction]."
            ],
            "type": {
              "option": "publicKey"
            }
          },
          {
            "name": "executedAt",
            "docs": [
              "If/when the transaction was executed."
            ],
            "type": {
              "option": "i64"
            }
          }
        ]
      }
    }
  ],
  "types": [
    {
      "name": "Instruction",
      "docs": [
        "Anchor (de-)serializable version of [solana_program::instruction::Instruction].",
        "Note that all member variable names are deliberately chosen to match the JS",
        "library, so that normal Solana instruction serialization should \"just work\" client-side."
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "programId",
            "docs": [
              "The program that this instruction invokes"
            ],
            "type": "publicKey"
          },
          {
            "name": "keys",
            "docs": [
              "[AccountMeta] listing which accounts are to be read/written-to during",
              "execution."
            ],
            "type": {
              "vec": {
                "defined": "AccountMeta"
              }
            }
          },
          {
            "name": "data",
            "docs": [
              "Serialized instruction data"
            ],
            "type": "bytes"
          }
        ]
      }
    },
    {
      "name": "AccountMeta",
      "docs": [
        "Anchor (de-)serializable version of [solana_program::instruction::AccountMeta]"
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "pubkey",
            "docs": [
              "An account that will be read or written to during transaction execution."
            ],
            "type": "publicKey"
          },
          {
            "name": "isSigner",
            "docs": [
              "True if an Instruction requires a Transaction signature matching `pubkey`."
            ],
            "type": "bool"
          },
          {
            "name": "isWritable",
            "docs": [
              "True if the `pubkey` can be loaded as a read-write account."
            ],
            "type": "bool"
          }
        ]
      }
    }
  ],
  "errors": [
    {
      "code": 6000,
      "name": "InvalidThreshold",
      "msg": "Threshold must be <= the number of multisig wallet members"
    },
    {
      "code": 6001,
      "name": "DuplicateMembers",
      "msg": "Members of a multisig must be unique addresses"
    },
    {
      "code": 6002,
      "name": "TooFewMembers",
      "msg": "Not enough members with the given threshold"
    },
    {
      "code": 6003,
      "name": "NotAMember",
      "msg": "Not a current member of the multisig wallet"
    },
    {
      "code": 6004,
      "name": "InvalidMemberSetSeqno",
      "msg": "The multisig wallet does not match the transaction's member_set_seqno"
    },
    {
      "code": 6005,
      "name": "InvalidMultisigReference",
      "msg": "The transaction does not belong to the provided multisig"
    },
    {
      "code": 6006,
      "name": "AlreadyApproved",
      "msg": "Signer already approved this transaction"
    },
    {
      "code": 6007,
      "name": "AlreadyUnapproved",
      "msg": "Signer already is marked as unapproved for this transaction."
    },
    {
      "code": 6008,
      "name": "NotEnoughApprovals",
      "msg": "Transaction requires more approvals before it can be executed"
    },
    {
      "code": 6009,
      "name": "AlreadyExecuted",
      "msg": "Transaction already executed"
    }
  ]
};

export const IDL: MultisigDemo = {
  "version": "0.1.0",
  "name": "multisig_demo",
  "instructions": [
    {
      "name": "newMultisig",
      "docs": [
        "Initialize a new [MultisigWallet]."
      ],
      "accounts": [
        {
          "name": "base",
          "isMut": false,
          "isSigner": true,
          "docs": [
            "Entropy to ensure a unique address.",
            "Improves on a design flaw in Project Serum's multisig implementation."
          ]
        },
        {
          "name": "payer",
          "isMut": true,
          "isSigner": true,
          "docs": [
            "Funds the creation of the wallet. Does not need to be contained in [members]."
          ]
        },
        {
          "name": "multisigWallet",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The PDA which stores relevant state, and which also signs approved transactions",
            "during their execution."
          ]
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "threshold",
          "type": "u16"
        },
        {
          "name": "members",
          "type": {
            "vec": "publicKey"
          }
        }
      ]
    },
    {
      "name": "newTransaction",
      "docs": [
        "Initialize a new [MultisigTransaction]."
      ],
      "accounts": [
        {
          "name": "proposer",
          "isMut": true,
          "isSigner": true,
          "docs": [
            "Must be a member of the given [multisig_wallet]."
          ]
        },
        {
          "name": "multisigWallet",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The wallet whose address is intended to sign the transaction",
            "upon execution. Only members of this wallet can sign for tx approval."
          ]
        },
        {
          "name": "transaction",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "instructions",
          "type": {
            "vec": {
              "defined": "Instruction"
            }
          }
        }
      ]
    },
    {
      "name": "approve",
      "docs": [
        "Approve a [MultisigTransaction] for execution."
      ],
      "accounts": [
        {
          "name": "member",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "multisigWallet",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "transaction",
          "isMut": true,
          "isSigner": false
        }
      ],
      "args": []
    },
    {
      "name": "unapprove",
      "docs": [
        "Cancel approval for a [MultisigTransaction]."
      ],
      "accounts": [
        {
          "name": "member",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "multisigWallet",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "transaction",
          "isMut": true,
          "isSigner": false
        }
      ],
      "args": []
    },
    {
      "name": "execute",
      "docs": [
        "Execute a [MultisigTransaction], iff it has enough approvals and hasn't",
        "yet been executed."
      ],
      "accounts": [
        {
          "name": "member",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "multisigWallet",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "transaction",
          "isMut": true,
          "isSigner": false
        }
      ],
      "args": []
    },
    {
      "name": "changeThreshold",
      "accounts": [
        {
          "name": "multisigWallet",
          "isMut": true,
          "isSigner": true
        }
      ],
      "args": [
        {
          "name": "threshold",
          "type": "u16"
        }
      ]
    },
    {
      "name": "changeMembers",
      "accounts": [
        {
          "name": "multisigWallet",
          "isMut": true,
          "isSigner": true
        }
      ],
      "args": [
        {
          "name": "members",
          "type": {
            "vec": "publicKey"
          }
        }
      ]
    }
  ],
  "accounts": [
    {
      "name": "multisigWallet",
      "docs": [
        "PDA that represents an M of N multisig signer, and all relevant metadata."
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "base",
            "docs": [
              "Base used to derive."
            ],
            "type": "publicKey"
          },
          {
            "name": "members",
            "docs": [
              "Members able to create and approve a [MultisigTransaction] owned by this",
              "multisig."
            ],
            "type": {
              "vec": "publicKey"
            }
          },
          {
            "name": "threshold",
            "docs": [
              "At least this many members needed to approve [MultisigTransaction]",
              "before it can be executed."
            ],
            "type": "u16"
          },
          {
            "name": "txNonce",
            "docs": [
              "To ensure uniqueness of [MultisigTransaction] PDA address."
            ],
            "type": "u64"
          },
          {
            "name": "memberSetSeqno",
            "docs": [
              "This is used to make sure that a transaction created with owner set X",
              "cannot be approved by a new owner set Y. This handles the edge cases",
              "where a user approves a transaction and then leaves multisig membership."
            ],
            "type": "u32"
          },
          {
            "name": "bump",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "multisigTransaction",
      "docs": [
        "PDA that represents a proposed transaction, and contains all relevant metadata."
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "instructions",
            "docs": [
              "The instruction set of the transaction to be executed."
            ],
            "type": {
              "vec": {
                "defined": "Instruction"
              }
            }
          },
          {
            "name": "multisigWallet",
            "docs": [
              "The [MultisigWallet] account this transaction belongs to."
            ],
            "type": "publicKey"
          },
          {
            "name": "approved",
            "docs": [
              "Keeps track of which accounts approved the transaction, and when",
              "their last approval occurred.",
              "`approved[i]` is Some iff `[MultisigWallet.members[i]]` signed the transaction."
            ],
            "type": {
              "vec": {
                "option": "i64"
              }
            }
          },
          {
            "name": "memberSetSeqno",
            "docs": [
              "Saved to ensure that no approvals occur on a different set of memberships."
            ],
            "type": "u32"
          },
          {
            "name": "createdAt",
            "docs": [
              "Unix timestamp at time of the [MultisigTransaction] account's creation."
            ],
            "type": "i64"
          },
          {
            "name": "proposer",
            "docs": [
              "The account that executed the [Transaction]."
            ],
            "type": "publicKey"
          },
          {
            "name": "executor",
            "docs": [
              "The account that executed the [Transaction]."
            ],
            "type": {
              "option": "publicKey"
            }
          },
          {
            "name": "executedAt",
            "docs": [
              "If/when the transaction was executed."
            ],
            "type": {
              "option": "i64"
            }
          }
        ]
      }
    }
  ],
  "types": [
    {
      "name": "Instruction",
      "docs": [
        "Anchor (de-)serializable version of [solana_program::instruction::Instruction].",
        "Note that all member variable names are deliberately chosen to match the JS",
        "library, so that normal Solana instruction serialization should \"just work\" client-side."
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "programId",
            "docs": [
              "The program that this instruction invokes"
            ],
            "type": "publicKey"
          },
          {
            "name": "keys",
            "docs": [
              "[AccountMeta] listing which accounts are to be read/written-to during",
              "execution."
            ],
            "type": {
              "vec": {
                "defined": "AccountMeta"
              }
            }
          },
          {
            "name": "data",
            "docs": [
              "Serialized instruction data"
            ],
            "type": "bytes"
          }
        ]
      }
    },
    {
      "name": "AccountMeta",
      "docs": [
        "Anchor (de-)serializable version of [solana_program::instruction::AccountMeta]"
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "pubkey",
            "docs": [
              "An account that will be read or written to during transaction execution."
            ],
            "type": "publicKey"
          },
          {
            "name": "isSigner",
            "docs": [
              "True if an Instruction requires a Transaction signature matching `pubkey`."
            ],
            "type": "bool"
          },
          {
            "name": "isWritable",
            "docs": [
              "True if the `pubkey` can be loaded as a read-write account."
            ],
            "type": "bool"
          }
        ]
      }
    }
  ],
  "errors": [
    {
      "code": 6000,
      "name": "InvalidThreshold",
      "msg": "Threshold must be <= the number of multisig wallet members"
    },
    {
      "code": 6001,
      "name": "DuplicateMembers",
      "msg": "Members of a multisig must be unique addresses"
    },
    {
      "code": 6002,
      "name": "TooFewMembers",
      "msg": "Not enough members with the given threshold"
    },
    {
      "code": 6003,
      "name": "NotAMember",
      "msg": "Not a current member of the multisig wallet"
    },
    {
      "code": 6004,
      "name": "InvalidMemberSetSeqno",
      "msg": "The multisig wallet does not match the transaction's member_set_seqno"
    },
    {
      "code": 6005,
      "name": "InvalidMultisigReference",
      "msg": "The transaction does not belong to the provided multisig"
    },
    {
      "code": 6006,
      "name": "AlreadyApproved",
      "msg": "Signer already approved this transaction"
    },
    {
      "code": 6007,
      "name": "AlreadyUnapproved",
      "msg": "Signer already is marked as unapproved for this transaction."
    },
    {
      "code": 6008,
      "name": "NotEnoughApprovals",
      "msg": "Transaction requires more approvals before it can be executed"
    },
    {
      "code": 6009,
      "name": "AlreadyExecuted",
      "msg": "Transaction already executed"
    }
  ]
};
