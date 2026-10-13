import '@testing-library/jest-dom/vitest';
import {vi,afterEach} from 'vitest';
import {cleanup} from '@testing-library/react';
window.NetLabNative={postMessage:vi.fn()};
window.matchMedia=vi.fn().mockImplementation(query=>({matches:false,media:query,addListener(){},removeListener(){},addEventListener(){},removeEventListener(){}}));
window.ResizeObserver=class{observe(){}unobserve(){}disconnect(){}};
afterEach(()=>{cleanup();localStorage.clear();vi.clearAllMocks();});
