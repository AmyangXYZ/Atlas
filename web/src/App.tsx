import { useEffect, useState } from "react"
import "./App.css"

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
}

function App() {
  const [chain, setChain] = useState<Block[]>([])
  const [peers, setPeers] = useState<[]>([])
  const [transaction, setTransaction] = useState(null)

  useEffect(() => {
    const ws = new WebSocket("ws://localhost:47100")
    ws.onopen = () => {
      console.log("Connected to server")
      ws.send("Hello, server!")
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
      if (data.type === "transaction") {
        setTransaction(data.value)
      }
    }
  }, [])

  return (
    <>
      <div>Chain: {JSON.stringify(chain)}</div>
      <div>Peers: {JSON.stringify(peers)}</div>
      <div>Transaction: {JSON.stringify(transaction)}</div>
    </>
  )
}

export default App
