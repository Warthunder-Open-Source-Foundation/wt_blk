from typing import Optional

class wt_blk_pybindings:
    """Python bindings for wt_blk"""
    def binary_blk_to_json(
        blk: bytes, dict: Optional[bytes] = None, nm: Optional[bytes] = None
    ) -> str: 
        """Converts a blk format binary block to a JSON string

        Args:
            blk (bytes): blk format binary block
            dict (Optional[bytes], optional): Defaults to None.
            nm (Optional[bytes], optional): Defaults to None.

        Returns:
            str: JSON string
        """
        ...
