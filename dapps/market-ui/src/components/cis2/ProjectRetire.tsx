import { ConcordiumGRPCClient, ContractAddress, TransactionStatusEnum } from "@concordium/web-sdk";
import { ContractInfo, connectToWallet } from "../../models/ConcordiumContractClient";
import { useState } from "react";
import { Button, Stack, TextField } from "@mui/material";
import DisplayError from "../ui/DisplayError";
import { retire } from "../../models/ProjectNFTClient";
import TransactionProgress from "../ui/TransactionProgress";

export function ProjectRetire(props: {
  grpcClient: ConcordiumGRPCClient;
  contractInfo: ContractInfo;
  address: ContractAddress;
  onDone: (output: { tokenIds: string[] }) => void;
}) {
  const [form, setForm] = useState({
    tokenId: "",
  });
  const [txn, setTxn] = useState<{ hash: string; status: TransactionStatusEnum }>();

  const [state, setState] = useState({
    isProcessing: false,
    error: "",
  });

  const onsubmit = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    setState({
      ...state,
      isProcessing: true,
      error: "",
    });
    setTxn(undefined);

    connectToWallet()
      .then((wallet) =>
        retire(
          wallet.provider,
          wallet.account,
          props.address,
          props.contractInfo,
          [form.tokenId],
          BigInt(9999),
          (status, hash) => setTxn({ status, hash }),
        ),
      )
      .then(() => {
        setState({ ...state, isProcessing: false });
        props.onDone({ tokenIds: [form.tokenId] });
      })
      .catch((e: Error) => {
        console.error(e);
        setState({ ...state, isProcessing: false, error: e.message });
      });
  };

  return (
    <>
      <Stack spacing={2} component={"form"} onSubmit={onsubmit}>
        <TextField
          label="Token ID"
          variant="outlined"
          fullWidth
          onChange={(e) => setForm({ ...form, tokenId: e.target.value })}
        />
        <DisplayError error={state.error} />
        <Button type="submit" variant="contained" color="primary">
          Retire
        </Button>
        {txn && <TransactionProgress hash={txn.hash} status={txn.status} />}
      </Stack>
    </>
  );
}