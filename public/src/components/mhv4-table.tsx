"use client";

import React from "react";
import { useMHV4Data } from "@/contexts/MHV4Context";

const NumberArrayComponent: React.FC = () => {
  const { voltageArray } = useMHV4Data();
  console.log("component" + voltageArray);

  return (
    <div>
      <h3>Number Array Data:</h3>
      <ul>
        {voltageArray.map((array, index) => (
          <li key={index}>{array.join(", ")}</li>
        ))}
      </ul>
    </div>
  );
};

export default NumberArrayComponent;
