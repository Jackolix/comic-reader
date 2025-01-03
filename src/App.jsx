import ComicReader from './components/ComicReader'
import { ThemeProvider } from 'next-themes'

function App({ Component, pageProps }) {
  return (
    <ThemeProvider attribute="class">
      <ComicReader />
    </ThemeProvider>
  )
}

export default App