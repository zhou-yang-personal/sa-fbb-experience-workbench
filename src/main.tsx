import React from 'react';
import ReactDOM from 'react-dom/client';
import { WorkbenchApp } from './features/workbench/WorkbenchApp';
import './styles.css';

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
    <WorkbenchApp />
  </React.StrictMode>,
);
