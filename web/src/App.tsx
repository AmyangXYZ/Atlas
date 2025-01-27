import { useEffect, useRef, useState } from "react"
import Stack from "@mui/material/Stack"
import "./App.css"
import Paper from "@mui/material/Paper"
import {
  Box,
  Table,
  TableCell,
  TableBody,
  TableContainer,
  TableRow,
  TableHead,
  Grid2 as Grid,
  Collapse,
  Typography,
  TableFooter,
  TablePagination,
  IconButton,
  AppBar,
  Toolbar,
} from "@mui/material"
import { createTheme, ThemeProvider } from "@mui/material/styles"
import { CssBaseline } from "@mui/material"
import { KeyboardArrowDown, KeyboardArrowUp, KeyboardDoubleArrowDown, Storage } from "@mui/icons-material"

const darkTheme = createTheme({
  palette: {
    mode: "dark",
    background: {
      default: "#0f1116",
      paper: "#0f1116",
    },
  },
})

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

type Cache = {
  name: string
  size: number
  last_updated: number
  last_accessed: number
  transactions: number
}

function App() {
  const [chain, setChain] = useState<Block[]>([])
  const [cache, setCache] = useState<Cache[]>([])
  const [showHistory, setShowHistory] = useState<string>("")
  const [history, setHistory] = useState<Transaction[]>([])
  const wsRef = useRef<WebSocket | null>(null)
  const chainDom = useRef<HTMLDivElement | null>(null)
  const getChain = () => {
    wsRef.current?.send(
      JSON.stringify({
        data: "chain",
      } as Query)
    )
  }

  const getHistory = (dataName: string) => {
    wsRef.current?.send(
      JSON.stringify({
        data: "history",
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
      if (data.type === "block") {
        setChain((prevChain) => [...prevChain, data.value])
      }
      if (data.type === "history") {
        const transactions = data.value.sort((a: Transaction, b: Transaction) => a.timestamp - b.timestamp)
        setHistory(transactions)
      }
      if (data.type === "cache") {
        setCache(data.value)
      }
    }
  }, [])

  useEffect(() => {
    if (chainDom.current) {
      chainDom.current.scrollTo({
        top: chainDom.current.scrollHeight,
      })
    }
  }, [chain])

  const [page, setPage] = useState(0)
  const rowsPerPage = 10
  const handleChangePage = (_event: unknown, newPage: number) => {
    setPage(newPage)
  }

  return (
    <ThemeProvider theme={darkTheme}>
      <CssBaseline />
      <Grid container spacing={4} sx={{ width: "1280px" }}>
        <Grid size={12}>
          <Box>
            <AppBar position="static" sx={{ borderRadius: "4px" }}>
              <Toolbar
                variant="dense"
                sx={{
                  display: "flex",
                  justifyContent: "space-between",
                  backgroundColor: "#1A2027",
                  borderRadius: "4px",
                }}
              >
                <Typography variant="h6" component="div">
                  IFT-ATLAS: Advanced Twin Linkage And Synchronization
                </Typography>

                <Typography variant="body1" component="div">
                  Nodes: 2 - Blocks: {chain.length} - Cached data: {cache.length}
                </Typography>
              </Toolbar>
            </AppBar>
          </Box>
        </Grid>
        <Grid size={4}>
          <Box
            ref={chainDom}
            sx={{
              width: "100%",
              height: "80vh",
              display: "flex",
              overflowY: "scroll",
              alignItems: "center",
              justifyContent: "center",
            }}
          >
            <Stack
              spacing={1}
              divider={
                <Box sx={{ display: "flex", alignItems: "center" }}>
                  <KeyboardDoubleArrowDown sx={{ margin: "auto", color: "#ccc" }} />
                </Box>
              }
            >
              {chain.map((block, i) => (
                <Box
                  key={i}
                  sx={{
                    width: "240px",
                    backgroundColor: "#1A2027",
                    padding: (theme) => theme.spacing(1),
                    textAlign: "center",
                    color: (theme) => theme.palette.text.secondary,
                    typography: "body2",
                    borderRadius: "4px",
                  }}
                >
                  <Stack spacing={1}>
                    <div
                      style={{
                        fontSize: "1.1rem",
                        fontWeight: "bold",
                        display: "flex",
                        alignItems: "center",
                        justifyContent: "center",
                        gap: "4px",
                      }}
                    >
                      <Storage />
                      Block #{i}
                    </div>
                    <div>Hash: {block.merkle_root.slice(0, 15)}...</div>
                    <div>Time: {formatDateTime(block.timestamp)}</div>
                    <div>Transactions: {block.transaction_count}</div>
                  </Stack>
                </Box>
              ))}
            </Stack>
          </Box>
        </Grid>
        <Grid size={8}>
          <Box sx={{ width: "100%", display: "flex", padding: 0, alignItems: "center" }}>
            <TableContainer component={Paper}>
              <Table size="small" sx={{ backgroundColor: "#1A2027" }}>
                <TableHead>
                  <TableRow
                    sx={{
                      "& .MuiTableCell-root": {
                        color: (theme) => theme.palette.text.secondary,
                        textAlign: "center",
                      },
                    }}
                  >
                    <TableCell>Name</TableCell>
                    <TableCell>Size</TableCell>
                    <TableCell>Last Updated</TableCell>
                    <TableCell>Last Accessed</TableCell>
                    <TableCell>Transactions</TableCell>
                  </TableRow>
                </TableHead>
                <TableBody>
                  {cache.slice(page * rowsPerPage, page * rowsPerPage + rowsPerPage).map((data, i) => (
                    <CacheRow
                      data={data}
                      history={history}
                      getHistory={getHistory}
                      showHistory={showHistory}
                      setShowHistory={setShowHistory}
                      key={i}
                    />
                  ))}
                </TableBody>
                <TableFooter>
                  <TableRow>
                    <TablePagination
                      colSpan={5}
                      count={cache.length}
                      rowsPerPage={rowsPerPage}
                      page={page}
                      onPageChange={handleChangePage}
                      rowsPerPageOptions={[]}
                    />
                  </TableRow>
                </TableFooter>
              </Table>
            </TableContainer>
          </Box>
        </Grid>
      </Grid>
    </ThemeProvider>
  )
}

function CacheRow({
  data,
  getHistory,
  history,
  showHistory,
  setShowHistory,
}: {
  data: Cache
  history: Transaction[]
  getHistory: (dataName: string) => void
  showHistory: string
  setShowHistory: (showHistory: string) => void
}) {
  const [page, setPage] = useState(0)
  const rowsPerPage = 5
  const handleChangePage = (_event: unknown, newPage: number) => {
    setPage(newPage)
  }

  return (
    <>
      <TableRow
        sx={{
          "& > *": { borderBottom: "unset" },
          "& .MuiTableCell-root": {
            color: (theme) => theme.palette.text.secondary,
            textAlign: "center",
          },
        }}
      >
        <TableCell align="center">{data.name}</TableCell>
        <TableCell align="center">{data.size}</TableCell>
        <TableCell align="center">{formatDateTime(data.last_updated)}</TableCell>
        <TableCell align="center">{formatDateTime(data.last_accessed)}</TableCell>
        <TableCell>
          {data.transactions}
          <IconButton
            aria-label="expand row"
            size="small"
            onClick={() => {
              setShowHistory(showHistory === data.name ? "" : data.name)
              if (showHistory !== data.name) {
                getHistory(data.name)
              }
            }}
          >
            {showHistory === data.name ? <KeyboardArrowUp /> : <KeyboardArrowDown />}
          </IconButton>
        </TableCell>
      </TableRow>
      <TableRow
        sx={{
          "& .MuiTableCell-root": {
            color: (theme) => theme.palette.text.secondary,
          },
        }}
      >
        <TableCell style={{ paddingBottom: 0, paddingTop: 0 }} colSpan={6}>
          <Collapse in={showHistory === data.name} timeout="auto" unmountOnExit>
            <Box sx={{ margin: 1 }}>
              <Typography variant="h6" gutterBottom component="div" sx={{ fontSize: "1.1rem" }}>
                Transactions
              </Typography>
            </Box>
            <Table size="small">
              <TableHead>
                <TableRow
                  sx={{
                    "& .MuiTableCell-root": {
                      color: (theme) => theme.palette.text.secondary,
                      textAlign: "center",
                    },
                  }}
                >
                  <TableCell>Client ID</TableCell>
                  <TableCell>Data Name</TableCell>
                  <TableCell>Operation</TableCell>
                  <TableCell>Timestamp</TableCell>
                </TableRow>
              </TableHead>
              <TableBody>
                {history.slice(page * rowsPerPage, page * rowsPerPage + rowsPerPage).map((transaction, i) => (
                  <TableRow
                    key={i}
                    sx={{
                      "& .MuiTableCell-root": { color: (theme) => theme.palette.text.secondary, textAlign: "center" },
                    }}
                  >
                    <TableCell>{transaction.client_id}</TableCell>
                    <TableCell>{transaction.data_name}</TableCell>
                    <TableCell>{transaction.operation}</TableCell>
                    <TableCell>{formatDateTime(transaction.timestamp)}</TableCell>
                  </TableRow>
                ))}
              </TableBody>
              <TableFooter>
                <TableRow>
                  <TablePagination
                    colSpan={5}
                    count={history.length}
                    rowsPerPage={rowsPerPage}
                    page={page}
                    onPageChange={handleChangePage}
                    rowsPerPageOptions={[]}
                  />
                </TableRow>
              </TableFooter>
            </Table>
          </Collapse>
        </TableCell>
      </TableRow>
    </>
  )
}

export default App

const formatDateTime = (timestamp: number): string => {
  if (timestamp === 0) return "N/A"
  return new Date(timestamp * 1000)
    .toLocaleString("en-US", {
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit",
      hour12: false,
      month: "numeric",
      day: "numeric",
      year: "numeric",
    })
    .replace(",", "")
}
