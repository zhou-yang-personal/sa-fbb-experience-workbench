import React from 'react';
import ReactDOM from 'react-dom/client';
import { WorkbenchAppV2 } from './features/workbench/WorkbenchAppV2';
import './styles.css';

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
    <WorkbenchAppV2 />
  </React.StrictMode>,
);
