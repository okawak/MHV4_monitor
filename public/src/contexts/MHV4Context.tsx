"use client";

import React, {
  createContext,
  useContext,
  useState,
  useEffect,
  ReactNode,
} from "react";

import {
  getInitRCStatus,
  getInitProgStatus,
  getInitMHV4bus,
  getInitMHV4dev,
  getInitMHV4ch,
  getInitMHV4onoff,
  getInitMHV4pol,
} from "@/lib/transformInitData";

type VoltageType = number[][];
type CurrentType = number[][];

interface MHV4ContextType {
  voltageArray: VoltageType;
  currentArray: CurrentType;
}

const defaultState: MHV4ContextType = {
  voltageArray: [
    [0, 0, 0, 0],
    [0, 0, 0, 0],
  ],
  currentArray: [[0, 0, 0, 0]],
};

const MHV4Context = createContext<MHV4ContextType>(defaultState);

interface MHV4ProviderProps {
  children: ReactNode;
}

export const MHV4DataProvider: React.FC<MHV4ProviderProps> = ({ children }) => {
  const [voltageArray, setVolArray] = useState<VoltageType>(
    defaultState.voltageArray,
  );
  const [currentArray, setCurArray] = useState<CurrentType>(
    defaultState.currentArray,
  );

  useEffect(() => {
    const fetchData = async () => {
      try {
        const response = await fetch(`${process.env.NEXT_PUBLIC_INIT_ROUTE}`);
        if (!response.ok) {
          throw new Error("failed to fetch the MHV4 data");
        }
        const initialData = await response.json();
        console.log(initialData);
        const data = JSON.parse(initialData);
        // process
        console.log(getInitRCStatus(data));
        console.log(getInitProgStatus(data));
        console.log(getInitMHV4bus(data));
        console.log(getInitMHV4dev(data));
        console.log(getInitMHV4ch(data));
        console.log(getInitMHV4onoff(data));
        console.log(getInitMHV4pol(data));
      } catch (error) {
        console.error("Failed to fetch initial data:", error);
      }
    };

    fetchData();

    const eventSource = new EventSource(`${process.env.NEXT_PUBLIC_SSE_ROUTE}`);
    eventSource.onopen = (event) => {
      console.log("SSE connection opened: ", event);
    };
    eventSource.onmessage = (event) => {
      console.log("SSE message received: ", event);
      const data = JSON.parse(event.data);
      // process
    };
    eventSource.onerror = (event) => {
      console.error("SSE connection error: ", event);
      // process
    };

    return () => {
      eventSource.close();
    };
  }, []);

  return (
    <MHV4Context.Provider value={{ voltageArray, currentArray }}>
      {children}
    </MHV4Context.Provider>
  );
};

export const useMHV4Data = (): MHV4ContextType => useContext(MHV4Context);
