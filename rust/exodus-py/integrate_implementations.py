#!/usr/bin/env python3
"""
Integrate all implementations from exomerge_implementations.py into exomerge.py
"""

import re

# Read the main file
with open('python/exodus/exomerge.py', 'r') as f:
    content = f.read()

# Track replacements
replacements = []

# =========================================================================
# 1. calculate_element_field_minimum
# =========================================================================
pattern = r'def calculate_element_field_minimum\(self, element_field_names.*?\):\s*""".*?"""\s*raise NotImplementedError\("calculate_element_field_minimum\(\) is not yet implemented\."\)'

replacement = '''def calculate_element_field_minimum(self, element_field_names: Union[str, List[str]],
                                       element_block_ids: Union[str, List[int]] = "all",
                                       timestep: Union[str, float] = "last") -> Union[float, Dict[str, float]]:
        """
        Find minimum value of element field(s).

        Parameters
        ----------
        element_field_names : str or list of str
            Element field name(s) to find minimum for
        element_block_ids : str or list of int, optional
            Element block IDs (default: "all")
        timestep : str or float, optional
            Timestep to use (default: "last")

        Returns
        -------
        float or dict
            Minimum value(s) of the field(s)
        """
        if element_block_ids == "all":
            element_block_ids = list(self.element_blocks.keys())
        elif isinstance(element_block_ids, int):
            element_block_ids = [element_block_ids]

        if isinstance(element_field_names, str):
            element_field_names = [element_field_names]

        # Get timestep index
        if timestep == "last":
            timestep_idx = len(self.timesteps) - 1 if self.timesteps else 0
        elif timestep == "first":
            timestep_idx = 0
        else:
            timestep_idx = min(range(len(self.timesteps)),
                             key=lambda i: abs(self.timesteps[i] - float(timestep)))

        min_values = {}
        for field_name in element_field_names:
            min_val = float('inf')
            for block_id in element_block_ids:
                if block_id in self.element_blocks:
                    fields = self.element_blocks[block_id][3]
                    if field_name in fields:
                        values = fields[field_name][timestep_idx]
                        if values:
                            min_val = min(min_val, min(values))
            min_values[field_name] = min_val if min_val != float('inf') else None

        return min_values if len(element_field_names) > 1 else min_values[element_field_names[0]]'''

if re.search(pattern, content, re.DOTALL):
    content = re.sub(pattern, replacement, content, flags=re.DOTALL)
    replacements.append("calculate_element_field_minimum")

# =========================================================================
# 2. create_averaged_element_field
# =========================================================================
pattern = r'def create_averaged_element_field\(self.*?\):\s*""".*?"""\s*raise NotImplementedError\("create_averaged_element_field\(\) is not yet implemented\."\)'

replacement = '''def create_averaged_element_field(self, element_field_names: Union[str, List[str]],
                                     new_field_name: str, element_block_id: int):
        """
        Create an element field by averaging existing fields.

        Parameters
        ----------
        element_field_names : str or list of str
            Element field name(s) to average
        new_field_name : str
            Name for the new averaged field
        element_block_id : int
            Element block ID

        Examples
        --------
        >>> model.create_averaged_element_field(["stress_1", "stress_2"], "avg_stress", 1)
        """
        if element_block_id not in self.element_blocks:
            self._error(f"Element block {element_block_id} does not exist")

        if isinstance(element_field_names, str):
            element_field_names = [element_field_names]

        # Verify all fields exist
        for field_name in element_field_names:
            if not self.element_field_exists(field_name, element_block_id):
                self._error(f"Element field '{field_name}' does not exist in block {element_block_id}")

        # Create new field
        self.create_element_field(new_field_name, element_block_id, 0.0)

        # Average the values
        fields = self.element_blocks[element_block_id][3]
        num_timesteps = len(self.timesteps) if self.timesteps else 1

        for timestep_idx in range(num_timesteps):
            num_elems = len(fields[element_field_names[0]][timestep_idx])
            new_values = [0.0] * num_elems

            for elem_idx in range(num_elems):
                total = 0.0
                for field_name in element_field_names:
                    total += fields[field_name][timestep_idx][elem_idx]
                new_values[elem_idx] = total / len(element_field_names)

            fields[new_field_name][timestep_idx] = new_values'''

if re.search(pattern, content, re.DOTALL):
    content = re.sub(pattern, replacement, content, flags=re.DOTALL)
    replacements.append("create_averaged_element_field")

# =========================================================================
# 3. convert_element_field_to_node_field
# =========================================================================
pattern = r'def convert_element_field_to_node_field\(self.*?\):\s*""".*?"""\s*raise NotImplementedError\("convert_element_field_to_node_field\(\) is not yet implemented\."\)'

replacement = '''def convert_element_field_to_node_field(self, element_field_name: str,
                                              node_field_name: Optional[str] = None,
                                              element_block_ids: Union[str, List[int]] = "all"):
        """
        Convert element field to node field by averaging element values at each node.

        Parameters
        ----------
        element_field_name : str
            Element field name to convert
        node_field_name : str, optional
            Name for node field (default: same as element_field_name)
        element_block_ids : str or list of int, optional
            Element block IDs (default: "all")

        Examples
        --------
        >>> model.convert_element_field_to_node_field("stress")
        >>> model.convert_element_field_to_node_field("stress", "node_stress", [1, 2])
        """
        if node_field_name is None:
            node_field_name = element_field_name

        if element_block_ids == "all":
            element_block_ids = list(self.element_blocks.keys())
        elif isinstance(element_block_ids, int):
            element_block_ids = [element_block_ids]

        # Create node field
        self.create_node_field(node_field_name, 0.0)

        num_timesteps = len(self.timesteps) if self.timesteps else 1

        for timestep_idx in range(num_timesteps):
            # Accumulate values and counts for each node
            node_sum = [0.0] * len(self.nodes)
            node_count = [0] * len(self.nodes)

            for block_id in element_block_ids:
                if block_id not in self.element_blocks:
                    continue

                fields = self.element_blocks[block_id][3]
                if element_field_name not in fields:
                    continue

                connectivity = self.get_connectivity(block_id)
                elem_values = fields[element_field_name][timestep_idx]

                for elem_idx, elem_nodes in enumerate(connectivity):
                    if elem_idx < len(elem_values):
                        elem_value = elem_values[elem_idx]
                        for node_idx in elem_nodes:
                            zero_based_idx = node_idx - 1
                            if 0 <= zero_based_idx < len(self.nodes):
                                node_sum[zero_based_idx] += elem_value
                                node_count[zero_based_idx] += 1

            # Calculate averages
            node_values = [
                node_sum[i] / node_count[i] if node_count[i] > 0 else 0.0
                for i in range(len(self.nodes))
            ]

            self.node_fields[node_field_name][timestep_idx] = node_values'''

if re.search(pattern, content, re.DOTALL):
    content = re.sub(pattern, replacement, content, flags=re.DOTALL)
    replacements.append("convert_element_field_to_node_field")

# =========================================================================
# 4. convert_node_field_to_element_field
# =========================================================================
pattern = r'def convert_node_field_to_element_field\(self.*?\):\s*""".*?"""\s*raise NotImplementedError\("convert_node_field_to_element_field\(\) is not yet implemented\."\)'

replacement = '''def convert_node_field_to_element_field(self, node_field_name: str,
                                              element_field_name: Optional[str] = None,
                                              element_block_ids: Union[str, List[int]] = "all"):
        """
        Convert node field to element field by averaging nodal values for each element.

        Parameters
        ----------
        node_field_name : str
            Node field name to convert
        element_field_name : str, optional
            Name for element field (default: same as node_field_name)
        element_block_ids : str or list of int, optional
            Element block IDs (default: "all")

        Examples
        --------
        >>> model.convert_node_field_to_element_field("temperature")
        >>> model.convert_node_field_to_element_field("temperature", "elem_temp", [1, 2])
        """
        if element_field_name is None:
            element_field_name = node_field_name

        if node_field_name not in self.node_fields:
            self._error(f"Node field '{node_field_name}' does not exist")

        if element_block_ids == "all":
            element_block_ids = list(self.element_blocks.keys())
        elif isinstance(element_block_ids, int):
            element_block_ids = [element_block_ids]

        num_timesteps = len(self.timesteps) if self.timesteps else 1

        for block_id in element_block_ids:
            if block_id not in self.element_blocks:
                continue

            # Create element field for this block
            self.create_element_field(element_field_name, block_id, 0.0)

            connectivity = self.get_connectivity(block_id)
            fields = self.element_blocks[block_id][3]

            for timestep_idx in range(num_timesteps):
                node_values = self.node_fields[node_field_name][timestep_idx]
                elem_values = []

                for elem_nodes in connectivity:
                    # Average node values for this element
                    total = 0.0
                    count = 0
                    for node_idx in elem_nodes:
                        zero_based_idx = node_idx - 1
                        if 0 <= zero_based_idx < len(node_values):
                            total += node_values[zero_based_idx]
                            count += 1
                    elem_value = total / count if count > 0 else 0.0
                    elem_values.append(elem_value)

                fields[element_field_name][timestep_idx] = elem_values'''

if re.search(pattern, content, re.DOTALL):
    content = re.sub(pattern, replacement, content, flags=re.DOTALL)
    replacements.append("convert_node_field_to_element_field")

# =========================================================================
# 5. calculate_node_field_maximum
# =========================================================================
pattern = r'def calculate_node_field_maximum\(self.*?\):\s*""".*?"""\s*raise NotImplementedError\("calculate_node_field_maximum\(\) is not yet implemented\."\)'

replacement = '''def calculate_node_field_maximum(self, node_field_names: Union[str, List[str]],
                                      timestep: Union[str, float] = "last") -> Union[float, Dict[str, float]]:
        """
        Calculate maximum value of node field(s).

        Parameters
        ----------
        node_field_names : str or list of str
            Node field name(s) to find maximum for
        timestep : str or float, optional
            Timestep to use (default: "last")

        Returns
        -------
        float or dict
            Maximum value(s) of the field(s)
        """
        if isinstance(node_field_names, str):
            node_field_names = [node_field_names]

        # Get timestep index
        if timestep == "last":
            timestep_idx = len(self.timesteps) - 1 if self.timesteps else 0
        elif timestep == "first":
            timestep_idx = 0
        else:
            timestep_idx = min(range(len(self.timesteps)),
                             key=lambda i: abs(self.timesteps[i] - float(timestep)))

        max_values = {}
        for field_name in node_field_names:
            if field_name in self.node_fields:
                values = self.node_fields[field_name][timestep_idx]
                max_values[field_name] = max(values) if values else None
            else:
                max_values[field_name] = None

        return max_values if len(node_field_names) > 1 else max_values[node_field_names[0]]'''

if re.search(pattern, content, re.DOTALL):
    content = re.sub(pattern, replacement, content, flags=re.DOTALL)
    replacements.append("calculate_node_field_maximum")

# =========================================================================
# 6. calculate_node_field_minimum
# =========================================================================
pattern = r'def calculate_node_field_minimum\(self.*?\):\s*""".*?"""\s*raise NotImplementedError\("calculate_node_field_minimum\(\) is not yet implemented\."\)'

replacement = '''def calculate_node_field_minimum(self, node_field_names: Union[str, List[str]],
                                      timestep: Union[str, float] = "last") -> Union[float, Dict[str, float]]:
        """
        Calculate minimum value of node field(s).

        Parameters
        ----------
        node_field_names : str or list of str
            Node field name(s) to find minimum for
        timestep : str or float, optional
            Timestep to use (default: "last")

        Returns
        -------
        float or dict
            Minimum value(s) of the field(s)
        """
        if isinstance(node_field_names, str):
            node_field_names = [node_field_names]

        # Get timestep index
        if timestep == "last":
            timestep_idx = len(self.timesteps) - 1 if self.timesteps else 0
        elif timestep == "first":
            timestep_idx = 0
        else:
            timestep_idx = min(range(len(self.timesteps)),
                             key=lambda i: abs(self.timesteps[i] - float(timestep)))

        min_values = {}
        for field_name in node_field_names:
            if field_name in self.node_fields:
                values = self.node_fields[field_name][timestep_idx]
                min_values[field_name] = min(values) if values else None
            else:
                min_values[field_name] = None

        return min_values if len(node_field_names) > 1 else min_values[node_field_names[0]]'''

if re.search(pattern, content, re.DOTALL):
    content = re.sub(pattern, replacement, content, flags=re.DOTALL)
    replacements.append("calculate_node_field_minimum")

# =========================================================================
# 7. displacement_field_exists
# =========================================================================
pattern = r'def displacement_field_exists\(self\):\s*""".*?"""\s*raise NotImplementedError\("displacement_field_exists\(\) is not yet implemented\."\)'

replacement = '''def displacement_field_exists(self) -> bool:
        """
        Check if displacement fields exist.

        Returns
        -------
        bool
            True if DISP_X, DISP_Y, and DISP_Z fields all exist
        """
        return (self.node_field_exists("DISP_X") and
                self.node_field_exists("DISP_Y") and
                self.node_field_exists("DISP_Z"))'''

if re.search(pattern, content, re.DOTALL):
    content = re.sub(pattern, replacement, content, flags=re.DOTALL)
    replacements.append("displacement_field_exists")

# =========================================================================
# 8. create_displacement_field
# =========================================================================
pattern = r'def create_displacement_field\(self.*?\):\s*""".*?"""\s*raise NotImplementedError\("create_displacement_field\(\) is not yet implemented\."\)'

replacement = '''def create_displacement_field(self, default_value: float = 0.0):
        """
        Create displacement fields (DISP_X, DISP_Y, DISP_Z).

        Parameters
        ----------
        default_value : float, optional
            Initial value for displacement fields (default: 0.0)

        Examples
        --------
        >>> model.create_displacement_field()
        >>> model.create_displacement_field(0.0)
        """
        self.create_node_field("DISP_X", default_value)
        self.create_node_field("DISP_Y", default_value)
        self.create_node_field("DISP_Z", default_value)'''

if re.search(pattern, content, re.DOTALL):
    content = re.sub(pattern, replacement, content, flags=re.DOTALL)
    replacements.append("create_displacement_field")

# =========================================================================
# 9. output_global_variables
# =========================================================================
pattern = r'def output_global_variables\(self.*?\):\s*""".*?"""\s*raise NotImplementedError\("output_global_variables\(\) is not yet implemented\."\)'

replacement = '''def output_global_variables(self, expressions: Union[Dict, List, str],
                                output_file: Optional[str] = None) -> str:
        """
        Output global variables to file or return as string.

        Parameters
        ----------
        expressions : dict, list, or str
            Variable names or expressions to output
        output_file : str, optional
            File path to write output (default: return as string)

        Returns
        -------
        str
            Tab-separated output of global variables

        Examples
        --------
        >>> output = model.output_global_variables(["energy", "volume"])
        >>> model.output_global_variables({"energy": "energy"}, "output.txt")
        """
        lines = []
        lines.append("# Global Variables")
        lines.append(f"# Timesteps: {len(self.timesteps)}")
        lines.append("")

        # Header
        header = ["Timestep"]
        if isinstance(expressions, dict):
            header.extend(expressions.keys())
            var_names = list(expressions.keys())
        elif isinstance(expressions, list):
            var_names = expressions
            header.extend(var_names)
        else:
            var_names = [expressions]
            header.extend(var_names)

        lines.append("\\t".join(header))

        # Data rows
        num_timesteps = len(self.timesteps) if self.timesteps else 1
        for timestep_idx in range(num_timesteps):
            row = [str(self.timesteps[timestep_idx]) if self.timesteps else str(timestep_idx)]
            for var_name in var_names:
                if var_name in self.global_variables:
                    values = self.global_variables[var_name]
                    if timestep_idx < len(values):
                        row.append(str(values[timestep_idx]))
                    else:
                        row.append("0.0")
                else:
                    row.append("0.0")
            lines.append("\\t".join(row))

        result = "\\n".join(lines)

        if output_file:
            with open(output_file, 'w') as f:
                f.write(result)

        return result'''

if re.search(pattern, content, re.DOTALL):
    content = re.sub(pattern, replacement, content, flags=re.DOTALL)
    replacements.append("output_global_variables")

# =========================================================================
# 10. create_interpolated_timestep
# =========================================================================
pattern = r'def create_interpolated_timestep\(self.*?\):\s*""".*?"""\s*raise NotImplementedError\("create_interpolated_timestep\(\) is not yet implemented\."\)'

replacement = '''def create_interpolated_timestep(self, timestep: float, interpolation: str = "linear"):
        """
        Create a new timestep by interpolating between existing timesteps.

        Parameters
        ----------
        timestep : float
            New timestep value
        interpolation : str, optional
            Interpolation method (default: "linear")

        Examples
        --------
        >>> model.create_interpolated_timestep(1.5)  # Interpolate at t=1.5
        """
        if not self.timesteps:
            self._error("No timesteps available for interpolation")

        timestep = float(timestep)

        # Check if timestep already exists
        if timestep in self.timesteps:
            self._warning(f"Timestep {timestep} already exists")
            return

        # Find surrounding timesteps
        lower_idx = None
        upper_idx = None

        for idx, ts in enumerate(self.timesteps):
            if ts < timestep:
                lower_idx = idx
            elif ts > timestep and upper_idx is None:
                upper_idx = idx
                break

        if lower_idx is None:
            # Before first timestep - use first two
            lower_idx = 0
            upper_idx = 1 if len(self.timesteps) > 1 else 0
        elif upper_idx is None:
            # After last timestep - use last two
            upper_idx = len(self.timesteps) - 1
            lower_idx = upper_idx - 1 if len(self.timesteps) > 1 else upper_idx

        t_lower = self.timesteps[lower_idx]
        t_upper = self.timesteps[upper_idx]

        # Calculate interpolation factor
        if t_upper == t_lower:
            factor = 0.5
        else:
            factor = (timestep - t_lower) / (t_upper - t_lower)

        # Insert timestep in sorted order
        insert_idx = upper_idx
        self.timesteps.insert(insert_idx, timestep)

        # Interpolate node fields
        for field_name, timestep_data in self.node_fields.items():
            lower_values = timestep_data[lower_idx]
            upper_values = timestep_data[upper_idx]
            interp_values = [
                lower_values[i] * (1 - factor) + upper_values[i] * factor
                for i in range(len(lower_values))
            ]
            timestep_data.insert(insert_idx, interp_values)

        # Interpolate global variables
        for var_name, timestep_data in self.global_variables.items():
            lower_value = timestep_data[lower_idx]
            upper_value = timestep_data[upper_idx]
            interp_value = lower_value * (1 - factor) + upper_value * factor
            timestep_data.insert(insert_idx, interp_value)

        # Interpolate element fields
        for block_id, (name, info, connectivity, fields) in self.element_blocks.items():
            for field_name, timestep_data in fields.items():
                lower_values = timestep_data[lower_idx]
                upper_values = timestep_data[upper_idx]
                interp_values = [
                    lower_values[i] * (1 - factor) + upper_values[i] * factor
                    for i in range(len(lower_values))
                ]
                timestep_data.insert(insert_idx, interp_values)

        # Interpolate side set fields
        for side_set_id, (name, members, fields) in self.side_sets.items():
            for field_name, timestep_data in fields.items():
                lower_values = timestep_data[lower_idx]
                upper_values = timestep_data[upper_idx]
                interp_values = [
                    lower_values[i] * (1 - factor) + upper_values[i] * factor
                    for i in range(len(lower_values))
                ]
                timestep_data.insert(insert_idx, interp_values)

        # Interpolate node set fields
        for node_set_id, (name, members, fields) in self.node_sets.items():
            for field_name, timestep_data in fields.items():
                lower_values = timestep_data[lower_idx]
                upper_values = timestep_data[upper_idx]
                interp_values = [
                    lower_values[i] * (1 - factor) + upper_values[i] * factor
                    for i in range(len(lower_values))
                ]
                timestep_data.insert(insert_idx, interp_values)'''

if re.search(pattern, content, re.DOTALL):
    content = re.sub(pattern, replacement, content, flags=re.DOTALL)
    replacements.append("create_interpolated_timestep")

# =========================================================================
# Save the file
# =========================================================================
with open('python/exodus/exomerge.py', 'w') as f:
    f.write(content)

# Count after
count_after = len(re.findall(r'raise NotImplementedError', content))

print(f"\\nIntegration complete!")
print(f"Methods integrated: {len(replacements)}")
for method in replacements:
    print(f"  ✓ {method}")
print(f"\\nNotImplementedError count: {count_before} → {count_after} (reduced by {count_before - count_after})")
