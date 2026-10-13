import React from 'react';
import {createRoot} from 'react-dom/client';
import {App} from './App.jsx';
class Boundary extends React.Component {state={failed:false};static getDerivedStateFromError(){return {failed:true};}render(){return this.state.failed?<div className="fallback"><h1>Vamos tentar de novo?</h1><p>A interface não abriu corretamente. Feche e abra o aplicativo. Se persistir, atualize o Android System WebView.</p><button onClick={()=>location.reload()}>Reabrir interface</button></div>:this.props.children;}}
createRoot(document.getElementById('root')).render(<Boundary><App/></Boundary>);
