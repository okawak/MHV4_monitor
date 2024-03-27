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

type OnoffType = boolean[][]; // true = ON
type PolType = boolean[][]; // true = positive
type VoltageType = number[][];
type CurrentType = number[][];

interface MHV4ContextType {
  onoffArray: OnoffType;
  polArray: PolType;
  voltageArray: VoltageType;
  currentArray: CurrentType;
}

const defaultState: MHV4ContextType = {
  onoffArray: [[false, false, false, false]],
  polArray: [[false, false, false, false]],
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
  const [onoffArray, setOnoffArray] = useState<OnoffType>(
    defaultState.onoffArray,
  );
  const [polArray, setPolArray] = useState<PolType>(defaultState.polArray);
  const [voltageArray, setVolArray] = useState<VoltageType>(
    defaultState.voltageArray,
  );
  const [currentArray, setCurArray] = useState<CurrentType>(
    defaultState.currentArray,
  );

  useEffect(() => {
    const fetchData = async () => {
      try {
        console.log(process.env.NEXT_PUBLIC_INIT_ROUTE);
        const response = await fetch(
          `${process.env.NEXT_PUBLIC_INIT_ROUTE}`,
        );
        if (!response.ok) {
          throw new Error("failed to fetch the MHV4 data");
        }
        const initialData = await response.json();
        console.log(initialData);
        // process
        console.log(getInitRCStatus(initialData));
        console.log(getInitProgStatus(initialData));
        console.log(getInitMHV4bus(initialData));
        console.log(getInitMHV4dev(initialData));
        console.log(getInitMHV4ch(initialData));
        console.log(getInitMHV4onoff(initialData));
        console.log(getInitMHV4pol(initialData));
      } catch (error) {
        console.error("Failed to fetch initial data:", error);
      }
    };

    fetchData();

    const eventSource = new EventSource(
      `${process.env.NEXT_PUBLIC_SSE_ROUTE}`,
    );
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
    <MHV4Context.Provider
      value={{ onoffArray, polArray, voltageArray, currentArray }}
    >
      {children}
    </MHV4Context.Provider>
  );
};

export const useMHV4Data = (): MHV4ContextType => useContext(MHV4Context);
