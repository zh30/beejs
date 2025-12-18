import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import { BrowserRouter, Routes, Route } from 'react-router-dom'
import RootLayout from './routes/__root'
import Home from './routes/index'
import Docs from './routes/docs'
import Blog from './routes/blog'
import './global.css'

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <BrowserRouter>
      <Routes>
        <Route element={<RootLayout />}>
          <Route path="/" element={<Home />} />
          <Route path="/docs" element={<Docs />}>
            <Route index element={<Docs />} />
            <Route path=":section" element={<Docs />} />
          </Route>
          <Route path="/blog" element={<Blog />}>
            <Route index element={<Blog />} />
            <Route path=":slug" element={<Blog />} />
          </Route>
        </Route>
      </Routes>
    </BrowserRouter>
  </StrictMode>,
)
