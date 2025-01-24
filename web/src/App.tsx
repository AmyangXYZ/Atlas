import { useEffect, useRef, useState } from "react"
import "./App.css"

type Query = {
  data: string
  params?: unknown
}

type Transaction = {
  client_id: number
  data_name: string
  operation: string
  timestamp: number
}

type Block = {
  merkle_root: string
  previous_hash: string
  timestamp: number
  transactions: Transaction[]
  transaction_count: number
}

function App() {
  const [chain, setChain] = useState<Block[]>([])
  const [peers, setPeers] = useState<[]>([])
  const [transactions, setTransactions] = useState<Transaction[]>([])
  const wsRef = useRef<WebSocket | null>(null)

  const getChain = () => {
    wsRef.current?.send(
      JSON.stringify({
        data: "chain",
      } as Query)
    )
  }

  const getPeers = () => {
    wsRef.current?.send(
      JSON.stringify({
        data: "peers",
      } as Query)
    )
  }

  const getBlock = (blockIndex: number) => {
    wsRef.current?.send(
      JSON.stringify({
        data: "block",
        params: blockIndex,
      } as Query)
    )
  }

  const getTransactions = (dataName: string) => {
    wsRef.current?.send(
      JSON.stringify({
        data: "transactions",
        params: dataName,
      } as Query)
    )
  }

  const getCache = () => {
    wsRef.current?.send(
      JSON.stringify({
        data: "cache",
      } as Query)
    )
  }

  useEffect(() => {
    const ws = new WebSocket("ws://localhost:47100")
    wsRef.current = ws
    ws.onopen = () => {
      console.log("Connected to server")
      getChain()
      getCache()
    }
    ws.onmessage = (event) => {
      const data = JSON.parse(event.data)
      if (data.type === "chain") {
        setChain(data.value)
      }
      if (data.type === "peers") {
        setPeers(data.value)
      }
      if (data.type === "block") {
        setChain((prevChain) => [...prevChain, data.value])
      }
      if (data.type === "transactions") {
        setTransactions((prevTransactions) => [...prevTransactions, data.value])
      }
    }
  }, [])

  return (
    <>
      <div>Chain: {JSON.stringify(chain)}</div>
      <div>Peers: {JSON.stringify(peers)}</div>
      <div>Transactions: {JSON.stringify(transactions)}</div>
    </>
  )
}

export default App
